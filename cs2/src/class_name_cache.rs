use std::collections::BTreeMap;

use anyhow::{
    anyhow,
    Context,
};
use cs2_schema_cutl::{
    CStringUtil,
    PtrCStr,
};
use raw_struct::{
    builtins::Ptr64,
    FromMemoryView,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CEntityIdentityEx,
    CS2Handle,
    StateCS2Handle,
    StateEntityList,
};

pub struct ClassNameCache {
    lookup: BTreeMap<u64, String>,
    reverse_lookup: BTreeMap<String, u64>,
}

impl State for ClassNameCache {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        Ok(Self {
            lookup: Default::default(),
            reverse_lookup: Default::default(),
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }

    fn update(&mut self, states: &StateRegistry) -> anyhow::Result<()> {
        let cs2 = states.resolve::<StateCS2Handle>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        for identity in entities.entities() {
            let class_info = identity.entity_class_info()?;
            self.register_class_info(&cs2, identity.entity_class_info()?)
                .with_context(|| {
                    format!(
                        "failed to generate class info for entity {:?} (class info = {:X})",
                        identity.handle::<()>().unwrap_or_default(),
                        class_info.address
                    )
                })?;
        }
        Ok(())
    }
}

impl ClassNameCache {
    fn register_class_info(
        &mut self,
        cs2: &CS2Handle,
        class_info: Ptr64<()>,
    ) -> anyhow::Result<()> {
        let address = class_info.address;
        if self.lookup.contains_key(&address) {
            /* we already know the name for this class */
            return Ok(());
        }

        let memory = cs2.create_memory_view();

        let class_name = PtrCStr::read_object(
            &*memory,
            u64::read_object(&*memory, address + 0x30).map_err(|e| anyhow!(e))? + 0x08,
        )
        .map_err(|e| anyhow!(e))?
        .read_string(&*memory)?
        .context("failed to read class name")?;

        self.lookup.insert(address, class_name.clone());
        self.reverse_lookup.insert(class_name, address);
        Ok(())
    }

    pub fn lookup(&self, class_info: &Ptr64<()>) -> anyhow::Result<Option<&String>> {
        let address = class_info.address;
        Ok(self.lookup.get(&address))
    }

    pub fn reverse_lookup(&self, name: &str) -> Option<u64> {
        self.reverse_lookup.get(name).cloned()
    }
}
