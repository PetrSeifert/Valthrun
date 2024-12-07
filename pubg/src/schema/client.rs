use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

#[raw_struct(size = 0x10256580)]
pub struct Globals {
    #[field(offset = 0x10256578)]
    pub u_world: Ptr64<dyn UWorld>,
}

#[raw_struct(size = 0x160)]
pub struct UWorld {
    #[field(offset = 0x158)]
    pub u_level: Ptr64<dyn ULevel>,
}

#[raw_struct(size = 0x70)]
pub struct ULevel {
    #[field(offset = 0x68)]
    pub actor_array: Ptr64<dyn ActorArray>,
}

#[raw_struct(size = 0x16)]
pub struct ActorArray {
    #[field(offset = 0x0)]
    pub data: u64,

    #[field(offset = 0x8)]
    pub count: u32,
}
