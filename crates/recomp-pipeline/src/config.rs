use crate::memory::{MemoryLayoutDescriptor, MemoryPermissionsDescriptor, MemoryRegionDescriptor};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StubBehavior {
    Log,
    Noop,
    Panic,
}

impl FromStr for StubBehavior {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "log" => Ok(StubBehavior::Log),
            "noop" => Ok(StubBehavior::Noop),
            "panic" => Ok(StubBehavior::Panic),
            other => Err(format!("unknown stub behavior: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    Handheld,
    Docked,
}

impl FromStr for PerformanceMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "handheld" => Ok(PerformanceMode::Handheld),
            "docked" => Ok(PerformanceMode::Docked),
            other => Err(format!("unknown performance mode: {other}")),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct RawRuntimeConfig {
    #[serde(default)]
    performance_mode: Option<String>,
    #[serde(default)]
    entry_address: Option<u64>,
    #[serde(default)]
    memory_layout: Option<RawMemoryLayoutConfig>,
}

#[derive(Debug, Deserialize)]
struct RawMemoryLayoutConfig {
    regions: Vec<RawMemoryRegionConfig>,
}

#[derive(Debug, Deserialize)]
struct RawMemoryRegionConfig {
    name: String,
    base: u64,
    size: u64,
    permissions: RawMemoryPermissionsConfig,
}

#[derive(Debug, Deserialize)]
struct RawMemoryPermissionsConfig {
    read: bool,
    write: bool,
    execute: bool,
}

#[derive(Debug)]
pub struct RuntimeConfig {
    pub performance_mode: PerformanceMode,
    pub entry_address: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct RawAssetsConfig {
    #[serde(default)]
    romfs_path: Option<PathBuf>,
    #[serde(default)]
    shader_cache_path: Option<PathBuf>,
    #[serde(default)]
    save_data_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
struct RawKeysConfig {
    #[serde(default)]
    prod_keys_path: Option<PathBuf>,
    #[serde(default)]
    title_keys_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
struct RawInputsConfig {
    #[serde(default)]
    xci_path: Option<PathBuf>,
    #[serde(default)]
    program_nca_path: Option<PathBuf>,
    #[serde(default)]
    exefs_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
struct RawPatchesConfig {
    #[serde(default)]
    patch_set: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
struct RawValidationConfig {
    #[serde(default)]
    reference_video_path: Option<PathBuf>,
    #[serde(default)]
    segment_start: Option<String>,
    #[serde(default)]
    segment_end: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawGpuConfig {
    #[serde(default)]
    backend: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTitleConfig {
    title: String,
    entry: String,
    abi_version: String,
    #[serde(default)]
    stubs: BTreeMap<String, String>,
    #[serde(default)]
    runtime: Option<RawRuntimeConfig>,
    #[serde(default)]
    assets: Option<RawAssetsConfig>,
    #[serde(default)]
    keys: Option<RawKeysConfig>,
    #[serde(default)]
    inputs: Option<RawInputsConfig>,
    #[serde(default)]
    patches: Option<RawPatchesConfig>,
    #[serde(default)]
    validation: Option<RawValidationConfig>,
    #[serde(default)]
    gpu: Option<RawGpuConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct AssetsConfig {
    pub romfs_path: Option<PathBuf>,
    pub shader_cache_path: Option<PathBuf>,
    pub save_data_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct KeysConfig {
    pub prod_keys_path: Option<PathBuf>,
    pub title_keys_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct InputsConfig {
    pub xci_path: Option<PathBuf>,
    pub program_nca_path: Option<PathBuf>,
    pub exefs_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct PatchesConfig {
    pub patch_set: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationConfig {
    pub reference_video_path: Option<PathBuf>,
    pub segment_start: Option<String>,
    pub segment_end: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GpuConfig {
    pub backend: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug)]
pub struct TitleConfig {
    pub title: String,
    pub entry: String,
    pub abi_version: String,
    pub stubs: BTreeMap<String, StubBehavior>,
    pub runtime: RuntimeConfig,
    pub memory_layout: MemoryLayoutDescriptor,
    pub assets: AssetsConfig,
    pub keys: KeysConfig,
    pub inputs: InputsConfig,
    pub patches: PatchesConfig,
    pub validation: ValidationConfig,
    pub gpu: GpuConfig,
}

impl TitleConfig {
    pub fn parse(toml_src: &str) -> Result<Self, String> {
        let raw: RawTitleConfig =
            toml::from_str(toml_src).map_err(|err| format!("invalid config: {err}"))?;
        let mut stubs = BTreeMap::new();
        for (name, behavior) in raw.stubs {
            let parsed = StubBehavior::from_str(&behavior)?;
            stubs.insert(name, parsed);
        }
        let runtime = raw.runtime.unwrap_or_default();
        let runtime_mode = runtime
            .performance_mode
            .unwrap_or_else(|| "handheld".to_string());
        let performance_mode = PerformanceMode::from_str(&runtime_mode)?;
        let memory_layout = match runtime.memory_layout {
            Some(layout) => parse_memory_layout(layout)?,
            None => {
                let layout = MemoryLayoutDescriptor::minimal_default();
                layout.validate()?;
                layout
            }
        };
        Ok(TitleConfig {
            title: raw.title,
            entry: raw.entry,
            abi_version: raw.abi_version,
            stubs,
            runtime: RuntimeConfig {
                performance_mode,
                entry_address: runtime.entry_address,
            },
            memory_layout,
            assets: parse_assets_config(raw.assets),
            keys: parse_keys_config(raw.keys),
            inputs: parse_inputs_config(raw.inputs),
            patches: parse_patches_config(raw.patches),
            validation: parse_validation_config(raw.validation),
            gpu: parse_gpu_config(raw.gpu),
        })
    }
}

fn parse_memory_layout(layout: RawMemoryLayoutConfig) -> Result<MemoryLayoutDescriptor, String> {
    let regions = layout
        .regions
        .into_iter()
        .map(|region| {
            MemoryRegionDescriptor::new(
                region.name,
                region.base,
                region.size,
                MemoryPermissionsDescriptor::new(
                    region.permissions.read,
                    region.permissions.write,
                    region.permissions.execute,
                ),
            )
        })
        .collect();
    let descriptor = MemoryLayoutDescriptor { regions };
    descriptor.validate()?;
    Ok(descriptor)
}

fn parse_assets_config(raw: Option<RawAssetsConfig>) -> AssetsConfig {
    let raw = raw.unwrap_or_default();
    AssetsConfig {
        romfs_path: raw.romfs_path,
        shader_cache_path: raw.shader_cache_path,
        save_data_dir: raw.save_data_dir,
    }
}

fn parse_keys_config(raw: Option<RawKeysConfig>) -> KeysConfig {
    let raw = raw.unwrap_or_default();
    KeysConfig {
        prod_keys_path: raw.prod_keys_path,
        title_keys_path: raw.title_keys_path,
    }
}

fn parse_inputs_config(raw: Option<RawInputsConfig>) -> InputsConfig {
    let raw = raw.unwrap_or_default();
    InputsConfig {
        xci_path: raw.xci_path,
        program_nca_path: raw.program_nca_path,
        exefs_dir: raw.exefs_dir,
    }
}

fn parse_patches_config(raw: Option<RawPatchesConfig>) -> PatchesConfig {
    let raw = raw.unwrap_or_default();
    PatchesConfig {
        patch_set: raw.patch_set,
    }
}

fn parse_validation_config(raw: Option<RawValidationConfig>) -> ValidationConfig {
    let raw = raw.unwrap_or_default();
    ValidationConfig {
        reference_video_path: raw.reference_video_path,
        segment_start: raw.segment_start,
        segment_end: raw.segment_end,
    }
}

fn parse_gpu_config(raw: Option<RawGpuConfig>) -> GpuConfig {
    let raw = raw.unwrap_or_default();
    GpuConfig {
        backend: raw.backend,
        notes: raw.notes,
    }
}
