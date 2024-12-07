use pubg::state::ActorArrayState;
use utils_state::StateRegistry;

pub struct PubgEnhancer {
    states: StateRegistry,
}

impl PubgEnhancer {
    pub fn new(states: StateRegistry) -> anyhow::Result<Self> {
        Ok(Self { states })
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        self.states.invalidate_states();

        let actor_array = self.states.resolve::<ActorArrayState>(())?;
        log::info!("Actor count: {}", actor_array.count);

        Ok(())
    }
}
