use anyhow::Context;
use clap::Parser;
use cs2::{
    CEntityIdentityEx,
    CS2Handle,
    CS2Model,
    ClassNameCache,
    LocalCameraControllerTarget,
    PlayerPawnState,
    StateBuildInfo,
    StateCS2Handle,
    StateCS2Memory,
    StateEntityList,
    StateLocalPlayerController,
    StatePawnInfo,
    StatePawnModelInfo,
};
use overlay_server::{OverlayServer, DrawCommand};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use utils_state::StateRegistry;

mod settings;
mod view_controller;

use view_controller::ViewController;
use settings::{AppSettings, EspBoxType, EspSelector};

#[derive(Parser)]
#[command(version)]
struct Args {
    /// Port for the overlay server
    #[arg(short = 'p', long, default_value = "8082")]
    port: u16,
}

struct PlayerInfo {
    position: nalgebra::Vector3<f32>,
    team_id: u8,
    health: i32,
    name: String,
    model_min: nalgebra::Vector3<f32>,
    model_max: nalgebra::Vector3<f32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logger - respects RUST_LOG env var, defaults to Info
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let args = Args::parse();
    
    println!("🎮 Valthrun GameBar Controller");
    println!("================================\n");

    // Initialize CS2 connection
    println!("📡 Connecting to CS2...");
    let cs2_handle = initialize_cs2().await?;
    
    println!("✅ Connected to CS2 (PID: {})\n", cs2_handle.process_id());

    // Initialize state registry
    let mut states = StateRegistry::new(1024 * 8);
    states.set(StateCS2Handle::new(cs2_handle.clone()), ())?;
    states.set(StateCS2Memory::new(cs2_handle.create_memory_view()), ())?;
    
    // Setup schema provider (required for CS2 offsets)
    println!("📋 Loading CS2 schema...");
    cs2_schema_provider_impl::setup_provider(Box::new(
        cs2_schema_provider_impl::RuntimeSchemaProvider::new(&states)
            .context("Failed to load runtime schema")?,
    ));
    
    // Verify CS2 version
    {
        let cs2_build_info = states.resolve::<StateBuildInfo>(())
            .context("Failed to load CS2 build info. CS2 version might be incompatible")?;
        
        println!("✅ Found Counter-Strike 2");
        println!("   Revision: {}", cs2_build_info.revision);
        println!("   Build: {}\n", cs2_build_info.build_datetime);
    } // Drop cs2_build_info here to release the borrow
    
    // Initialize view controller for coordinate transformation
    log::debug!("Initializing view controller...");
    states.resolve::<ViewController>(())?;
    log::debug!("View controller ready");
    
    // Start overlay server
    let bind_addr = format!("0.0.0.0:{}", args.port);
    println!("🚀 Starting overlay server on {}...", bind_addr);
    let server = OverlayServer::bind(&bind_addr).await?;
    
    // Spawn server.run() in background to accept connections
    let server_run = server.clone();
    tokio::spawn(async move {
        if let Err(e) = server_run.run().await {
            log::error!("Server error: {}", e);
        }
    });
    
    println!("✅ Overlay server started!");
    println!("\n📋 Waiting for GameBar overlay widget to connect...");
    println!("   Connect your widget to: 127.0.0.1:{}\n", args.port);
    
    // Wait for widget connection
    while server.client_count().await == 0 {
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("✅ Widget connected!\n");
    println!("🎯 Starting ESP...\n");
    
    // Create a channel for sending draw commands with high priority
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<DrawCommand>>();
    let server_for_sender = server.clone();
    
    // Spawn a dedicated high-priority sender task
    tokio::spawn(async move {
        while let Some(commands) = rx.recv().await {
            let send_start = Instant::now();
            if let Err(e) = server_for_sender.send_batch(commands).await {
                log::warn!("Failed to send batch: {}", e);
            }
            let send_time = send_start.elapsed();
            log::debug!("[Sender] Actual send took {:?}", send_time);
        }
    });
    
    // Load settings (use defaults for now)
    let settings = AppSettings::default();
    
    // Main loop
    let mut last_update = Instant::now();
    let mut frame_count = 0u64;
    let target_fps = 60; // 60 FPS is sufficient for ESP (16.6ms per frame)
    let frame_time = Duration::from_millis(1000 / target_fps);
    let mut last_send_time = Instant::now();
    
    loop {
        let loop_start = Instant::now();
        let now = Instant::now();
        let delta = now.duration_since(last_update);
        
        if delta < frame_time {
            let sleep_time = frame_time - delta;
            sleep(sleep_time).await;
            continue;
        }
        
        let frame_start = Instant::now();
        last_update = now;
        frame_count += 1;
        
        log::trace!("=== Frame {} ===", frame_count);
        
        // Invalidate states to force updates (view matrix, player positions, etc.)
        let invalidate_start = Instant::now();
        states.invalidate_states();
        let invalidate_time = invalidate_start.elapsed();
        
        // Render ESP
        let render_start = Instant::now();
        if let Err(e) = render_esp(&tx, &states, &settings).await {
            log::debug!("Render error: {}", e);
        }
        let render_time = render_start.elapsed();
        
        // Track time since last send
        let send_interval = last_send_time.elapsed();
        last_send_time = Instant::now();
        
        let frame_time_elapsed = frame_start.elapsed();
        let loop_time = loop_start.elapsed();
        
        // Log if frame takes longer than expected or send interval is too long
        if frame_time_elapsed.as_millis() > frame_time.as_millis() || send_interval.as_millis() > 20 {
            log::debug!("[Frame {}] Slow frame: total={:?}, invalidate={:?}, render={:?}, loop={:?}, send_interval={:?}", 
                frame_count, frame_time_elapsed, invalidate_time, render_time, loop_time, send_interval);
        }
        
        // Check if widget is still connected
        if server.client_count().await == 0 {
            println!("⚠️ Widget disconnected, waiting for reconnection...");
            while server.client_count().await == 0 {
                sleep(Duration::from_millis(500)).await;
            }
            println!("✅ Widget reconnected!");
        }
    }
}

async fn initialize_cs2() -> anyhow::Result<Arc<CS2Handle>> {
    // CS2Handle::create now automatically finds the CS2 process
    // It takes a metrics flag (false for now)
    CS2Handle::create(false)
        .context("Failed to create CS2 handle. Is CS2 running?")
}

async fn render_esp(
    tx: &mpsc::UnboundedSender<Vec<DrawCommand>>,
    states: &StateRegistry,
    settings: &AppSettings,
) -> anyhow::Result<()> {
    let frame_start = Instant::now();
    log::trace!("render_esp() called");
    
    // Get required states (these will update because we called invalidate_states())
    let state_start = Instant::now();
    let memory = states.resolve::<StateCS2Memory>(())?;
    let entities = states.resolve::<StateEntityList>(())?;
    let class_name_cache = states.resolve::<ClassNameCache>(())?;
    let view_controller = states.resolve::<ViewController>(())?;
    log::trace!("ViewController resolved for this frame (took {:?})", state_start.elapsed());
    
    // Collect all draw commands to send in one batch
    let mut commands = Vec::new();
    
    // Get local player info
    let local_player_controller = states.resolve::<StateLocalPlayerController>(())?;
    let Some(local_player_controller) = local_player_controller
        .instance
        .value_reference(memory.view_arc())
    else {
        return Ok(());
    };
    
    let local_team_id = local_player_controller.m_iPendingTeamNum()?;
    
    // Get view target
    let view_target = states.resolve::<LocalCameraControllerTarget>(())?;
    let view_target_entity_id = match &view_target.target_entity_id {
        Some(value) => *value,
        None => return Ok(()),
    };
    
    // Collect all players
    let mut players = Vec::new();
    
    for entity_identity in entities.entities() {
        // Skip local player
        if entity_identity.handle::<()>()?.get_entity_index() == view_target_entity_id {
            continue;
        }
        
        // Check if entity is a player pawn
        let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info()?)?;
        if !entity_class
            .map(|name| *name == "C_CSPlayerPawn")
            .unwrap_or(false)
        {
            continue;
        }
        
        // Check if player is alive
        let pawn_state = states.resolve::<PlayerPawnState>(entity_identity.handle()?)?;
        if *pawn_state != PlayerPawnState::Alive {
            continue;
        }
        
        // Get player info
        let pawn_info = states.resolve::<StatePawnInfo>(entity_identity.handle()?)?;
        
        if pawn_info.player_health <= 0 || pawn_info.player_name.is_none() {
            continue;
        }
        
        // Get model info
        let pawn_model = states.resolve::<StatePawnModelInfo>(entity_identity.handle()?)?;
        let model = states.resolve::<CS2Model>(pawn_model.model_address)?;
        
        players.push(PlayerInfo {
            position: pawn_info.position,
            team_id: pawn_info.team_id,
            health: pawn_info.player_health,
            name: pawn_info.player_name.clone().unwrap_or_default(),
            model_min: model.vhull_min,
            model_max: model.vhull_max,
        });
    }
    
    let player_collect_time = state_start.elapsed();
    log::trace!("Found {} players to render (collection took {:?})", players.len(), player_collect_time);
    
    // Draw ESP for each player
    for (idx, player) in players.iter().enumerate() {
        log::trace!("Rendering player {}/{}: team={}, health={}, pos=({:.1}, {:.1}, {:.1}), is_enemy={}", 
            idx + 1, players.len(), player.team_id, player.health, 
            player.position.x, player.position.y, player.position.z,
            player.team_id != local_team_id);
        // Determine if enemy
        let is_enemy = player.team_id != local_team_id;
        
        // Get ESP settings
        let esp_selector = EspSelector::PlayerTeamVisibility {
            enemy: is_enemy,
            visible: true,
        };
        
        let esp_settings = settings.get_esp_player_settings(&esp_selector);
        if esp_settings.is_none() {
            log::trace!("  No ESP settings for this player, skipping");
            continue;
        }
        let esp_settings = esp_settings.unwrap();
        
        // Calculate 2D box from 3D bounds
        let box_2d = view_controller.calculate_box_2d(
            &(player.model_min + player.position),
            &(player.model_max + player.position),
        );
        
        if let Some((min, max)) = box_2d {
            log::trace!("  Box 2D calculated: min=({:.3}, {:.3}), max=({:.3}, {:.3})", 
                min.x, min.y, max.x, max.y);
            // Calculate color based on health
            let health_percent = (player.health as f32 / 100.0).clamp(0.0, 1.0);
            let color = calculate_esp_color(&esp_settings.box_color, health_percent);
            
            // Draw box (as 4 individual lines for proper batching)
            match esp_settings.box_type {
                EspBoxType::Box2D | EspBoxType::Box3D => {
                    let width = max.x - min.x;
                    let height = max.y - min.y;
                    let line_width = esp_settings.box_width as f64;
                    
                    // Top line
                    commands.push(DrawCommand::line(
                        min.x as f64, min.y as f64,
                        (min.x + width) as f64, min.y as f64,
                        &color, line_width
                    ));
                    // Right line
                    commands.push(DrawCommand::line(
                        (min.x + width) as f64, min.y as f64,
                        (min.x + width) as f64, (min.y + height) as f64,
                        &color, line_width
                    ));
                    // Bottom line
                    commands.push(DrawCommand::line(
                        (min.x + width) as f64, (min.y + height) as f64,
                        min.x as f64, (min.y + height) as f64,
                        &color, line_width
                    ));
                    // Left line
                    commands.push(DrawCommand::line(
                        min.x as f64, (min.y + height) as f64,
                        min.x as f64, min.y as f64,
                        &color, line_width
                    ));
                }
                EspBoxType::None => {}
            }
            
            // Draw health bar (left side of box) - simple line version
            if esp_settings.health_bar_enabled {
                let bar_width = esp_settings.health_bar_width / 1000.0; // Scale to normalized coords
                let bar_height = (max.y - min.y) * health_percent;
                let x = min.x - bar_width - esp_settings.box_width / 1000.0;
                let y = max.y - bar_height;
                
                // Health bar color based on health
                let health_color = if health_percent > 0.6 {
                    "#00FF00" // Green
                } else if health_percent > 0.3 {
                    "#FFFF00" // Yellow
                } else {
                    "#FF0000" // Red
                };
                
                // Draw health bar as thick line
                commands.push(DrawCommand::line(
                    x as f64,
                    y as f64,
                    x as f64,
                    max.y as f64,
                    health_color,
                    bar_width as f64,
                ));
            }
        } else {
            log::trace!("  Box not visible on screen, skipping");
        }
    }
    
    // Send batch via channel (non-blocking, immediate)
    let send_start = Instant::now();
    if !commands.is_empty() {
        log::trace!("Sending batch of {} draw commands", commands.len());
        if let Err(e) = tx.send(commands) {
            log::warn!("Failed to send batch to channel: {}", e);
        }
    } else {
        // If no players visible, send empty batch to clear screen
        log::trace!("No players visible, sending empty batch to clear");
        if let Err(e) = tx.send(vec![]) {
            log::warn!("Failed to send empty batch to channel: {}", e);
        }
    }
    let send_time = send_start.elapsed();
    log::debug!("[Send] Channel send time={:?}", send_time);
    
    let total_time = frame_start.elapsed();
    log::debug!("Frame timing: states={:?}, players={:?}, send={:?}, TOTAL={:?}", 
        state_start.elapsed(), player_collect_time, send_time, total_time);
    
    Ok(())
}

fn calculate_esp_color(esp_color: &settings::EspColor, health_percent: f32) -> String {
    use settings::EspColor;
    
    match esp_color {
        EspColor::Static { value } => {
            let rgba = value.as_u8();
            format!("#{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2])
        }
        EspColor::HealthBased { max, mid, min } => {
            // Interpolate between colors based on health
            if health_percent > 0.5 {
                let t = (health_percent - 0.5) * 2.0;
                interpolate_color(mid, max, t)
            } else {
                let t = health_percent * 2.0;
                interpolate_color(min, mid, t)
            }
        }
        EspColor::HealthBasedRainbow { .. } => {
            // Simple rainbow based on health
            if health_percent > 0.6 {
                "#00FF00".to_string() // Green
            } else if health_percent > 0.3 {
                "#FFFF00".to_string() // Yellow
            } else {
                "#FF0000".to_string() // Red
            }
        }
        EspColor::DistanceBased { .. } => {
            // For now, use static color
            "#FFFFFF".to_string()
        }
    }
}

fn interpolate_color(from: &settings::Color, to: &settings::Color, t: f32) -> String {
    let from_rgba = from.as_f32();
    let to_rgba = to.as_f32();
    
    let r = (from_rgba[0] + (to_rgba[0] - from_rgba[0]) * t) * 255.0;
    let g = (from_rgba[1] + (to_rgba[1] - from_rgba[1]) * t) * 255.0;
    let b = (from_rgba[2] + (to_rgba[2] - from_rgba[2]) * t) * 255.0;
    
    format!("#{:02X}{:02X}{:02X}", r as u8, g as u8, b as u8)
}

