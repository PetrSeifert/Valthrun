use std::{
    fs::File,
    io::Write,
    mem,
};

use raw_struct::{Reference, FromMemoryView};

use anyhow::Context;
use pubg::{
    decrypt::StateDecrypt,
    schema::{
        AActor,
        APlayerController,
    },
    state::{
        StateActorList,
        StateLocalPlayerInfo,
        StatePlayerInfo,
    },
    Module,
    StatePubgHandle,
    StatePubgMemory,
};

use super::Enhancement;

pub struct PlayerSpyer {
    //init: bool,
}

impl PlayerSpyer {
    pub fn new() -> Self {
        Self { 
            //init: false 
        }
    }
    
    /// XOR keys used for player health decryption
    const HEALTH_XOR_KEYS: [u32; 16] = [
        0xCEC7A59F,
        0x9B63B23E,
        0xCAF75ABD,
        0x3E38486F,
        0xE8911D0A,
        0x23DDAD1C,
        0x9456FC8,
        0xBD39B621,
        0xBAD7A58,
        0xA8EF3E87,
        0xE2752BB6,
        0x9F8ADBF4,
        0xBDE8FBD5,
        0x3E936F07,
        0x6F099E38,
        0xE2D72AE4
    ];

    const HEALTH: u32 = 0xA30;
    const HEALTH_FLAG: u32 = 0x380;
    const HEALTH1: u32 = 0xA28;
    const HEALTH2: u32 = 0x970;
    const HEALTH3: u32 = 0xA44;
    const HEALTH4: u32 = 0xA30;
    const HEALTH5: u32 = 0xA45;
    const HEALTH6: u32 = 0xA40;

    pub fn decrypt_player_health(&self, value: &mut [u8], offset: u32) {
        let xor_keys = unsafe { std::slice::from_raw_parts((&Self::HEALTH_XOR_KEYS as *const u32) as *const u8, 64) };
        let size = value.len() as u32;
        for i in 0..size as usize {
            value[i] ^= xor_keys[(i as u32 + offset) as usize & 0x3F];
        }
    }

    pub fn get_health(&self, actor: Reference<dyn AActor>, ctx: &crate::UpdateContext) -> anyhow::Result<f32> {
        let memory = ctx.states.resolve::<StatePubgMemory>(())?;
        let b_health_flag = u8::read_object(memory.view(), actor.reference_address() + Self::HEALTH_FLAG as u64).map_err(|err| anyhow::anyhow!("{}", err))? != 3;
        let b_health1 = u32::read_object(memory.view(), actor.reference_address() + Self::HEALTH1 as u64).map_err(|err| anyhow::anyhow!("{}", err))? != 0;
        if b_health_flag && b_health1 {
            let b_is_encrypted = u8::read_object(memory.view(), actor.reference_address() + Self::HEALTH5 as u64).map_err(|err| anyhow::anyhow!("{}", err))? != 0;
            let health3 = u8::read_object(memory.view(), actor.reference_address() + Self::HEALTH3 as u64).map_err(|err| anyhow::anyhow!("{}", err))?; 
            let mut health4 = f32::read_object(memory.view(), health3 as u64 + actor.reference_address() + Self::HEALTH4 as u64).map_err(|err| anyhow::anyhow!("{}", err))?;

            if b_is_encrypted {
                let mut health = health4.to_le_bytes();
                self.decrypt_player_health(&mut health, u32::read_object(memory.view(), actor.reference_address() + Self::HEALTH6 as u64).map_err(|err| anyhow::anyhow!("{}", err))?);
                health4 = f32::from_le_bytes(health);
            }

            Ok(health4)
        }
        else {
            Ok(f32::read_object(memory.view(), actor.reference_address() + Self::HEALTH2 as u64).map_err(|err| anyhow::anyhow!("{}", err))?)
        }
    }
}

impl Enhancement for PlayerSpyer {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let memory = ctx.states.resolve::<StatePubgMemory>(())?;

        /*if !self.init {
            self.init = true;
            let handle = ctx.states.resolve::<StatePubgHandle>(())?;

            let module_size = handle.module_size(Module::Game)?;
            let module_address = handle.memory_address(Module::Game, 0x0)?;
            log::info!("Module size: {:#x}", module_size);
            log::info!("Module address: {:#x}", module_address);

            let mut buffer = vec![0u8; 0x17474000 as usize];
            let mut start = 0;
            let mut end = 0x1C1C0;
            for i in 0..3392  {
                log::info!("Reading module {}/3392", i+1);
                let mut temp_buffer = vec![0u8; 0x1C1C0 as usize];
                match memory.read_memory(module_address + 0x1000 + (i * 0x1C1C0), &mut temp_buffer) {
                    Ok(()) => {
                        buffer[start..end].copy_from_slice(&temp_buffer);
                    },
                    Err(err) => {
                        log::error!("Failed to read module: {}", err);
                        continue;
                    }
                }
                start = end;
                end += 0x1C1C0;
            }

            let mut file = File::create(format!("dump1.bin"))?;
            file.write_all(&buffer)?;

            log::info!("Done");
        }*/

        let actor_array = ctx.states.resolve::<StateActorList>(())?;
        let decrypt = ctx.states.resolve::<StateDecrypt>(())?;
        let local_player_info = ctx.states.resolve::<StateLocalPlayerInfo>(())?;
        let actor_count = actor_array.count()?;
        
        let mut players_data: Vec<(u32, u32, i32)> = Vec::new();

        for actor_ptr in actor_array
            .data()?
            .elements(memory.view(), 0..actor_count as usize)?
        {
            if actor_ptr.is_null() {
                continue;
            }

            let actor = actor_ptr
                .value_reference(memory.view_arc())
                .context("actor nullptr")?;

            let root_component = match actor
                .root_component()?
                .value_reference(memory.view_arc(), &decrypt)
            {
                Some(root_component) => root_component,
                None => {
                    continue;
                }
            };

            let player_controller = actor.cast::<dyn APlayerController>();

            if player_controller.player_state()?.is_null() {
                /* Actor is not a player controller */
                continue;
            }

            if !player_controller.acknowledged_pawn()?.is_null() {
                continue;
            }

            if player_controller.reference_address() == local_player_info.controller_address {
                log::info!("Skipping local player");
                continue;
            }

            let mesh = match player_controller.mesh()?.read_value(memory.view()) {
                Ok(mesh) => mesh,
                _ => {
                    continue;
                }
            };

            let mesh = match mesh {
                Some(mesh) => mesh,
                None => {
                    continue;
                }
            };

            if mesh <= 0x10000 {
                continue;
            }

            let health = self.get_health(actor, ctx)?;
            if health <= 1.0 || health > 101.0 {
                continue;
            }
            let health = health as u32;

            let relative_location = root_component.relative_location()?;
            let distance = ((relative_location[0] - local_player_info.location[0]).powi(2) +
                            (relative_location[1] - local_player_info.location[1]).powi(2) +
                            (relative_location[2] - local_player_info.location[2]).powi(2))
                .sqrt() as u32;

            let difference = [
                relative_location[0] - local_player_info.location[0],
                relative_location[1] - local_player_info.location[1],
                relative_location[2] - local_player_info.location[2],
            ];
            
            // Calculate horizontal angle to target (atan2 gives us angle in radians)
            let target_angle = difference[1].atan2(difference[0]);
            
            // Get player's horizontal angle (z rotation)
            let player_angle = local_player_info.rotation[1].to_radians();
            
            // Calculate the difference and convert to degrees
            let angle_diff = (target_angle - player_angle).to_degrees();
            
            // Normalize angle to -180 to 180 degrees
            let angle_diff = if angle_diff > 180.0 {
                (angle_diff - 360.0) as i32
            } else if angle_diff < -180.0 {
                (angle_diff + 360.0) as i32
            } else {
                angle_diff as i32
            };
            
            players_data.push((distance, health, angle_diff));
        }

        players_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut count = players_data.len();
        if count > 25 {
            count = 25;
        }
        for i in 0..count {
            log::info!("Distance: {} Health: {} Angle: {}", players_data[i].0, players_data[i].1, players_data[i].2);
        }

        Ok(())
    }
}
