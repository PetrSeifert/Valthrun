use std::ops::Deref;

use anyhow::Context;
use raw_struct::{
    builtins::Ptr64,
    Copy,
    Reference,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    decrypt::StateDecrypt,
    schema::{
        AActor,
        Entry,
        TArray,
        ENTRY_OFFSET,
    },
    Module,
    StatePubgHandle,
    StatePubgMemory,
};

pub struct StateActorList(Copy<dyn TArray<Ptr64<dyn AActor>>>);
impl State for StateActorList {
    type Parameter = ();

    fn create(states: &StateRegistry, _: Self::Parameter) -> anyhow::Result<Self> {
        let handle = states.resolve::<StatePubgHandle>(())?;
        let memory = states.resolve::<StatePubgMemory>(())?;
        let decrypt = states.resolve::<StateDecrypt>(())?;

        let base_address = handle.memory_address(Module::Game, 0x0)?;
        let entry = Reference::<dyn Entry>::new(memory.clone(), base_address + ENTRY_OFFSET);
        let u_world = entry
            .u_world()
            .context("u_world nullptr")?
            .value_reference(memory.view_arc(), &decrypt)
            .context("nullptr")?;

        let u_level = u_world
            .u_level()
            .context("u_level nullptr")?
            .value_reference(memory.view_arc(), &decrypt)
            .context("nullptr")?;

        let actor_array = u_level
            .actors()
            .context("actor_array nullptr")?
            .value_copy(memory.view(), &decrypt)?
            .context("nullptr")?;

        Ok(Self(actor_array))
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

impl Deref for StateActorList {
    type Target = Copy<dyn TArray<Ptr64<dyn AActor>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
