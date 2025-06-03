//! Display various information about `cargo gpu`, eg its cache directory.

use std::process::{Command, Stdio};

use crate::cache_dir;

/// Show the computed source of the spirv-std dependency.
#[derive(Clone, Debug, clap::Parser)]
pub struct SpirvSourceDep {
    /// The location of the shader-crate to inspect to determine its spirv-std dependency.
    #[clap(long, default_value = "./")]
    pub shader_crate: std::path::PathBuf,
}

/// Different tidbits of information that can be queried at the command line.
#[derive(Clone, Debug, clap::Subcommand)]
pub enum Info {
    /// Displays the location of the cache directory
    CacheDirectory,
    /// The source location of spirv-std
    SpirvSource(SpirvSourceDep),
    /// The git commitsh of this cli tool.
    Commitsh,
    /// All the available SPIR-V capabilities that can be set with `--capabilities`
    Capabilities,

    /// All available SPIR-V targets
    Targets,
}

/// `cargo gpu show`
#[derive(clap::Parser)]
pub struct Show {
    /// Display information about rust-gpu
    #[clap(subcommand)]
    command: Info,
}

impl Show {
    /// Entrypoint
    pub fn run(&self) -> anyhow::Result<()> {
        log::info!("{:?}: ", self.command);

        #[expect(
            clippy::print_stdout,
            reason = "The output of this command could potentially be used in a script, \
                      so we _don't_ want to use `crate::user_output`, as that prefixes a crab."
        )]
        match &self.command {
            Info::CacheDirectory => {
                println!("{}\n", cache_dir()?.display());
            }
            Info::SpirvSource(SpirvSourceDep { shader_crate }) => {
                let rust_gpu_source =
                    crate::spirv_source::SpirvSource::get_rust_gpu_deps_from_shader(shader_crate)?;
                println!("{rust_gpu_source}\n");
            }
            Info::Commitsh => {
                println!("{}", env!("GIT_HASH"));
            }
            Info::Capabilities => {
                println!("All available options to the `cargo gpu build --capabilities` argument:");
                #[expect(
                    clippy::use_debug,
                    reason = "It's easier to just use `Debug` formatting than implementing `Display`"
                )]
                for capability in Self::capability_variants_iter() {
                    println!("  {capability:?}");
                }
            }
            Info::Targets => {
                let target_info = get_spirv_targets()?.join("\n");
                println!("{}", target_info);
            }
        }

        Ok(())
    }

    /// Iterator over all `Capability` variants.
    fn capability_variants_iter() -> impl Iterator<Item = spirv_builder::Capability> {
        // Since spirv::Capability is repr(u32) we can iterate over
        // u32s until some maximum
        #[expect(clippy::as_conversions, reason = "We know all variants are repr(u32)")]
        let last_capability = spirv_builder::Capability::CacheControlsINTEL as u32;
        (0..=last_capability).filter_map(spirv_builder::Capability::from_u32)
    }
}

/// Gets available SPIR-V targets by calling the `spirv-tools`' validator:
/// ```sh
/// $ spirv-val --version
/// SPIRV-Tools v2022.2-dev unknown hash, 2022-02-16T16:37:15
/// Targets:
///   SPIR-V 1.0
///   SPIR-V 1.1
///   SPIR-V 1.2
///   ... snip for brevity
///  SPIR-V 1.6 (under Vulkan 1.3 semantics)
///  ```
fn get_spirv_targets() -> anyhow::Result<Vec<String>> {
    // Defaults that have been tested, 1.2 is the existing default in the shader-crate-template.toml
    let mut targets = vec![
        "spirv-unknown-vulkan1.0",
        "spirv-unknown-vulkan1.1",
        "spirv-unknown-vulkan1.2",
    ];

    let output = Command::new("spirv-val")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    if let Ok(output) = output {
        let version_info = String::from_utf8_lossy(&output.stdout);
        if version_info.contains("SPIR-V 1.3") {
            targets.push("spirv-unknown-vulkan1.3");
        }
        if version_info.contains("SPIR-V 1.4") {
            targets.push("spirv-unknown-vulkan1.4");
        }
        // Exhaustively, manually put in all possible versions? or regex them out?
    }

    Ok(targets.into_iter().map(String::from).collect())
}
