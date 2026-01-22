use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Color representation
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Color(u32);

impl Color {
    pub fn as_u8(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub fn as_f32(&self) -> [f32; 4] {
        self.as_u8()
            .map(|channel| (channel as f32) / (u8::MAX as f32))
    }

    pub const fn from_u8(value: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(value))
    }

    pub const fn from_f32(value: [f32; 4]) -> Self {
        Self::from_u8([
            (value[0] * 255.0) as u8,
            (value[1] * 255.0) as u8,
            (value[2] * 255.0) as u8,
            (value[3] * 255.0) as u8,
        ])
    }
}

impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Self {
        Self::from_u8(value)
    }
}

impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Self {
        Self::from_f32(value)
    }
}

/// ESP color modes
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
#[serde(tag = "type", content = "options")]
pub enum EspColor {
    HealthBasedRainbow { alpha: f32 },
    HealthBased { max: Color, mid: Color, min: Color },
    Static { value: Color },
    DistanceBased { near: Color, mid: Color, far: Color },
}

impl Default for EspColor {
    fn default() -> Self {
        Self::Static {
            value: Color::from_f32([1.0, 1.0, 1.0, 1.0]),
        }
    }
}

/// ESP box type
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum EspBoxType {
    Box2D,
    Box3D,
    None,
}

impl Default for EspBoxType {
    fn default() -> Self {
        Self::Box2D
    }
}

/// ESP player settings
#[derive(Clone, Deserialize, Serialize)]
pub struct EspPlayerSettings {
    pub box_type: EspBoxType,
    pub box_color: EspColor,
    pub box_width: f32,
    
    pub health_bar_enabled: bool,
    pub health_bar_width: f32,
    
    pub show_name: bool,
    pub show_weapon: bool,
    pub show_distance: bool,
}

impl Default for EspPlayerSettings {
    fn default() -> Self {
        Self {
            box_type: EspBoxType::Box2D,
            box_color: EspColor::HealthBased {
                max: Color::from_f32([0.0, 1.0, 0.0, 1.0]), // Green
                mid: Color::from_f32([1.0, 1.0, 0.0, 1.0]), // Yellow
                min: Color::from_f32([1.0, 0.0, 0.0, 1.0]), // Red
            },
            box_width: 2.0,
            health_bar_enabled: true,
            health_bar_width: 4.0,
            show_name: true,
            show_weapon: false,
            show_distance: false,
        }
    }
}

/// ESP selector for different player states
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
pub enum EspSelector {
    PlayerTeamVisibility { enemy: bool, visible: bool },
}

impl EspSelector {
    pub fn config_key(&self) -> String {
        match self {
            EspSelector::PlayerTeamVisibility { enemy, visible } => {
                format!("player_{}_{}", 
                    if *enemy { "enemy" } else { "team" },
                    if *visible { "visible" } else { "invisible" }
                )
            }
        }
    }
}

/// Application settings
#[derive(Clone, Deserialize, Serialize)]
pub struct AppSettings {
    pub esp_settings: HashMap<String, EspPlayerSettings>,
    pub esp_settings_enabled: HashMap<String, bool>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut settings = AppSettings {
            esp_settings: HashMap::new(),
            esp_settings_enabled: HashMap::new(),
        };
        
        // Enable ESP for enemies (visible)
        let enemy_key = EspSelector::PlayerTeamVisibility { 
            enemy: true, 
            visible: true 
        }.config_key();
        
        settings.esp_settings.insert(
            enemy_key.clone(),
            EspPlayerSettings {
                box_color: EspColor::HealthBased {
                    max: Color::from_f32([0.0, 1.0, 0.0, 1.0]), // Green
                    mid: Color::from_f32([1.0, 1.0, 0.0, 1.0]), // Yellow
                    min: Color::from_f32([1.0, 0.0, 0.0, 1.0]), // Red
                },
                ..Default::default()
            },
        );
        settings.esp_settings_enabled.insert(enemy_key, true);
        
        // Enable ESP for teammates (visible) with different color
        let team_key = EspSelector::PlayerTeamVisibility { 
            enemy: false, 
            visible: true 
        }.config_key();
        
        settings.esp_settings.insert(
            team_key.clone(),
            EspPlayerSettings {
                box_color: EspColor::Static {
                    value: Color::from_f32([0.0, 0.5, 1.0, 1.0]), // Blue
                },
                ..Default::default()
            },
        );
        settings.esp_settings_enabled.insert(team_key, true);
        
        settings
    }
}

impl AppSettings {
    pub fn get_esp_player_settings(&self, selector: &EspSelector) -> Option<&EspPlayerSettings> {
        let config_key = selector.config_key();
        
        if self.esp_settings_enabled.get(&config_key).cloned().unwrap_or(false) {
            self.esp_settings.get(&config_key)
        } else {
            None
        }
    }
}

