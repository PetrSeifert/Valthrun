use anyhow::Context;
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    Module,
    Signature,
    StateCS2Handle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CS2Offset {
    Globals,
    BuildInfo,

    LocalController,
    GlobalEntityList,

    ViewMatrix,
    NetworkGameClientInstance,

    CCVars,
    SchemaSystem,
}

impl CS2Offset {
    pub fn available_offsets() -> &'static [Self] {
        &[
            CS2Offset::Globals,
            CS2Offset::BuildInfo,
            CS2Offset::LocalController,
            CS2Offset::GlobalEntityList,
            CS2Offset::ViewMatrix,
            CS2Offset::NetworkGameClientInstance,
            CS2Offset::CCVars,
            CS2Offset::SchemaSystem,
        ]
    }

    pub fn cache_name(&self) -> &'static str {
        match self {
            Self::Globals => "globals",
            Self::BuildInfo => "build-info",
            Self::LocalController => "local-controller",
            Self::GlobalEntityList => "global-entity-list",
            Self::ViewMatrix => "view-matrix",
            Self::NetworkGameClientInstance => "network-game-client-instance",
            Self::CCVars => "ccvars",
            Self::SchemaSystem => "schema-system",
        }
    }

    pub fn signature(&self) -> (Module, Signature) {
        match *self {
            Self::Globals => (
                Module::Client,
                Signature::relative_address(
                    obfstr!("client globals"),
                    obfstr!("48 8B 05 ? ? ? ? 8B 48 04 FF C1"),
                    0x03,
                    0x07,
                ),
            ),
            Self::BuildInfo => (
                Module::Engine,
                Signature::relative_address(
                    obfstr!("client build info"),
                    obfstr!("48 8B 1D ? ? ? ? 48 8D 3D"),
                    0x03,
                    0x07,
                ),
            ),
            Self::LocalController => (
                Module::Client,
                Signature::relative_address(
                    obfstr!("local player controller ptr"),
                    obfstr!("48 83 3D ? ? ? ? ? 0F 95"),
                    0x03,
                    0x08,
                ),
            ),
            Self::GlobalEntityList => (
                Module::Client,
                Signature::relative_address(
                    obfstr!("global entity list"),
                    obfstr!("4C 8B 0D ? ? ? ? 48 89 5C 24 ? 8B"),
                    0x03,
                    0x07,
                ),
            ),
            Self::ViewMatrix => (
                Module::Client,
                Signature::relative_address(
                    obfstr!("world view matrix"),
                    obfstr!("48 8D 0D ? ? ? ? 48 C1 E0 06"),
                    0x03,
                    0x07,
                ),
            ),
            Self::NetworkGameClientInstance => (
                Module::Engine,
                Signature::relative_address(
                    obfstr!("network game client instance"),
                    obfstr!("48 83 3D ? ? ? ? ? 48 8B D9 8B 0D"),
                    0x03,
                    0x08,
                ),
            ),
            Self::CCVars => (
                Module::Tier0,
                Signature::relative_address(
                    obfstr!("CCVars"),
                    obfstr!("4C 8D 2D ? ? ? ? 0F 28 45"),
                    0x03,
                    0x07,
                ),
            ),
            Self::SchemaSystem => (
                Module::Schemasystem,
                Signature::relative_address(
                    obfstr!("schema system instance"),
                    obfstr!("48 8B 0D ? ? ? ? 48 8B 55 A0"),
                    0x03,
                    0x07,
                ),
            ),
        }
    }
}

pub struct StatePredefinedOffset {
    pub module: Module,
    pub offset: u64,
    pub resolved: u64,
}

impl StatePredefinedOffset {
    pub fn new(states: &StateRegistry, offset: CS2Offset, value: u64) -> anyhow::Result<Self> {
        let cs2 = states.resolve::<StateCS2Handle>(())?;

        let (module, _) = offset.signature();
        let resolved = cs2.memory_address(module, value)?;

        Ok(Self {
            module,
            offset: value,
            resolved,
        })
    }
}

impl State for StatePredefinedOffset {
    type Parameter = CS2Offset;

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

pub struct StateResolvedOffset {
    pub offset: u64,
    pub address: u64,
}

impl State for StateResolvedOffset {
    type Parameter = CS2Offset;

    fn create(states: &StateRegistry, offset: Self::Parameter) -> anyhow::Result<Self> {
        let cs2 = states.resolve::<StateCS2Handle>(())?;
        let (module, signature) = offset.signature();

        if let Some(offset) = states.get::<StatePredefinedOffset>(offset) {
            /* use predefined value */
            Ok(Self {
                offset: offset.offset,
                address: offset.resolved,
            })
        } else {
            /* resolve at runtime */
            let address = cs2
                .resolve_signature(module, &signature)
                .with_context(|| format!("offset {:?}", offset))?;

            let offset = cs2
                .module_address(module, address)
                .context("resolved signature is not contained in module")?;

            Ok(Self { offset, address })
        }
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}
