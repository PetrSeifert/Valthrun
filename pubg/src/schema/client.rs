use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

use crate::EncryptedPtr64;

pub const ENTRY_OFFSET: u64 = 0x108535F8;

#[raw_struct(size = 0x8)]
pub struct Entry {
    #[field(offset = 0x0)]
    pub u_world: EncryptedPtr64<dyn UWorld>,
}

#[raw_struct(size = 0x168)]
pub struct UWorld {
    #[field(offset = 0x150)]
    pub u_level: EncryptedPtr64<dyn ULevel>,

    #[field(offset = 0x160)]
    pub game_instance: EncryptedPtr64<dyn GameInstance>,
}

#[raw_struct(size = 0x48)]
pub struct ULevel {
    #[field(offset = 0x40)]
    pub actors: EncryptedPtr64<dyn TArray<Ptr64<dyn AActor>>>,
}

#[raw_struct(size = 0x80)]
pub struct GameInstance {
    #[field(offset = 0x78)]
    pub local_players: EncryptedPtr64<dyn LocalPlayers>,
}

#[raw_struct(size = 0x38)]
pub struct LocalPlayers {
    #[field(offset = 0x30)]
    pub player_controller: EncryptedPtr64<dyn APlayerController>,
}

#[raw_struct(size = 0x10)]
pub struct TArray<T>
where
    T: Send + Sync + 'static,
{
    #[field(offset = 0x0)]
    pub data: Ptr64<[T]>,

    #[field(offset = 0x8)]
    pub count: u32,

    #[field(offset = 0xC)]
    pub max: u32,
}

#[raw_struct(size = 0x478)]
pub struct AActor {
    #[field(offset = 0x220)]
    pub root_component: Ptr64<dyn USceneComponent>,

    #[field(offset = 0x470)]
    pub mesh: Ptr64<u64>,
}

#[raw_struct(size = 0x4C8)]
pub struct APlayerController {
    #[field(offset = 0x428)]
    pub player_state: Ptr64<()>,

    #[field(offset = 0x498)]
    pub acknowledged_pawn: Ptr64<dyn APawn>,

    #[field(offset = 0x4C0)]
    pub player_camera_manager: Ptr64<dyn APlayerCameraManager>,
}
impl AActor for dyn APlayerController {}

#[raw_struct(size = 0x1744)]
pub struct APawn {
    #[field(offset = 0x13B0)]
    pub spectated_count: u32,

    #[field(offset = 0x1740)]
    pub last_team_num: u32,
}
impl AActor for dyn APawn {}

#[raw_struct(size = 0x464)]
pub struct APlayerCameraManager {
    #[field(offset = 0x458)]
    pub camera_pos: [f32; 3],
}
impl AActor for dyn APlayerCameraManager {}

#[raw_struct(size = 0xF8)]
pub struct USceneComponent {
    #[field(offset = 0xF0)]
    pub relative_location: [f32; 3],
}
