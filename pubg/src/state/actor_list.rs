use anyhow::Context;
use raw_struct::Reference;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    decrypt::StateDecrypt,
    handle::StatePubgHandle,
    schema::Globals,
    Module::Game,
    StatePubgMemory,
};

pub struct ActorArrayState {
    pub data_ptr: u64,
    pub count: u32,
}

impl State for ActorArrayState {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let handle = states.resolve::<StatePubgHandle>(())?;
        let memory = states.resolve::<StatePubgMemory>(())?;
        let decrypt = states.resolve::<StateDecrypt>(())?;

        let base_address = handle.memory_address(Game, 0x0)?;
        log::info!("Base address: {}", base_address);
        let globals = Reference::<dyn Globals>::new(memory.clone(), base_address);
        let mut u_world_ptr = globals.u_world()?;
        unsafe {
            u_world_ptr.address = decrypt.decrypt(u_world_ptr.address);
        }
        log::info!("UWorld: {}", u_world_ptr.address);
        let u_world = u_world_ptr
            .value_reference(memory.view_arc())
            .context("nullptr")?;
        let mut u_level_ptr = u_world.u_level()?;
        unsafe {
            u_level_ptr.address = decrypt.decrypt(u_level_ptr.address);
        }
        log::info!("ULevel: {}", u_level_ptr.address);
        let _u_level = u_level_ptr
            .value_reference(memory.view_arc())
            .context("nullptr")?;
        //let actor_array = u_level.actor_array()?.value_reference(memory.view_arc()).context("nullptr")?;

        Ok(Self {
            data_ptr: 0,
            count: 0,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
