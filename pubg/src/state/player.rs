use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use super::StateActorList;

#[derive(Debug, Clone)]
pub struct StatePlayerInfo {
    pub position: [f32; 3],
}

impl State for StatePlayerInfo {
    type Parameter = u32;

    fn create(states: &StateRegistry, actor_index: Self::Parameter) -> anyhow::Result<Self> {
        let actor_list = states.resolve::<StateActorList>(())?;

        Ok(Self { position: [0.0; 3] })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
