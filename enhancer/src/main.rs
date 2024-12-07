use enhancer::PubgEnhancer;
use pubg::{
    InterfaceError,
    PubgHandle,
    StatePubgHandle,
    StatePubgMemory,
};
use utils_state::StateRegistry;

mod enhancer;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let mut pubg_enhancer = {
        let cs2 = match PubgHandle::create(true) {
            Ok(cs2) => cs2,
            Err(err) => {
                if let Some(err) = err.downcast_ref::<InterfaceError>() {
                    if let Some(detailed_message) = err.detailed_message() {
                        for line in detailed_message.lines() {
                            log::error!("{}", line);
                        }
                        return Ok(());
                    }
                }

                return Err(err);
            }
        };
        let mut states = StateRegistry::new(1024 * 8);
        states.set(StatePubgMemory::new(cs2.create_memory_view()), ())?;
        states.set(StatePubgHandle::new(cs2), ())?;

        Box::new(PubgEnhancer::new(states)?)
    };

    loop {
        match pubg_enhancer.update() {
            Ok(_) => {}
            Err(err) => {
                log::error!("{}", err)
            }
        }
    }
}
