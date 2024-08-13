use ripgrep_all::{
    adapters::{get_all_adapters, AdapterMeta},
    config::RgaConfig,
};

pub fn adapters(config: RgaConfig) -> Vec<AdapterMeta> {
    let (enabled_adapters, disabled_adapters) = get_all_adapters(config.custom_adapters);

    enabled_adapters
        .iter()
        .chain(disabled_adapters.iter())
        .map(|adapter| adapter.metadata())
        .cloned()
        .collect()
}
