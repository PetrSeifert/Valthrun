use anyhow::Context;
use pubg::{
    schema::{
        AActor,
        APlayerController,
    },
    state::{
        StateActorList,
        StatePlayerInfo,
    },
    StatePubgMemory,
};

use super::Enhancement;

struct PlayerSpyerInfo {
    name: String,
    level: u32,
}

pub struct PlayerSpyer {}

impl PlayerSpyer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Enhancement for PlayerSpyer {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let memory = ctx.states.resolve::<StatePubgMemory>(())?;
        let actor_array = ctx.states.resolve::<StateActorList>(())?;
        let actor_count = actor_array.count()?;
        log::info!("Actor count: {}", actor_count);

        for actor_ptr in actor_array
            .data()?
            .elements(memory.view(), 0..actor_count as usize)?
        {
            let actor = actor_ptr
                .value_reference(memory.view_arc())
                .context("nullptr")?;

            let player_controller = actor.cast::<dyn APlayerController>();

            if player_controller.player_state()?.is_null() {
                /* Actor is not a player controller */
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

            if mesh < 0xFFFF {
                continue;
            }

            log::info!("Player mesh: {}", mesh);
        }

        Ok(())
    }
}
