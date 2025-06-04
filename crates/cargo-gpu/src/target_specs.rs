//! Legacy target specs are spec jsons for versions before `rustc_codegen_spirv-target-specs`
//! came bundled with them. Instead, cargo gpu needs to bundle these legacy spec files. Luckily,
//! they are the same for all versions, as bundling target specs with the codegen backend was
//! introduced before the first target spec update.

use crate::cache_dir;
use crate::spirv_source::{FindPackage as _, SpirvSource};
use anyhow::Context as _;
use cargo_metadata::Metadata;
use std::path::{Path, PathBuf};

/// Extract legacy target specs from our executable into some directory
pub fn write_legacy_target_specs(target_spec_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(target_spec_dir)?;
    for (filename, contents) in legacy_target_specs::TARGET_SPECS {
        let path = target_spec_dir.join(filename);
        std::fs::write(&path, contents.as_bytes())
            .with_context(|| format!("writing legacy target spec file at [{}]", path.display()))?;
    }
    Ok(())
}

/// Copy spec files from one dir to another, assuming no subdirectories
fn copy_spec_files(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    let dir = std::fs::read_dir(src)?;
    for dir_entry in dir {
        let file = dir_entry?;
        let file_path = file.path();
        if file_path.is_file() {
            std::fs::copy(file_path, dst.join(file.file_name()))?;
        }
    }
    Ok(())
}

/// Computes the `target-specs` directory to use and updates the target spec files, if enabled.
pub fn update_target_specs_files(
    source: &SpirvSource,
    dummy_metadata: &Metadata,
    update_files: bool,
) -> anyhow::Result<PathBuf> {
    log::info!(
        "target-specs: Resolving target specs `{}`",
        if update_files {
            "and update them"
        } else {
            "without updating"
        }
    );

    let mut target_specs_dst = source.install_dir()?.join("target-specs");
    if let Ok(target_specs) = dummy_metadata.find_package("rustc_codegen_spirv-target-specs") {
        log::info!(
            "target-specs: found crate `rustc_codegen_spirv-target-specs` with manifest at `{}`",
            target_specs.manifest_path
        );

        let target_specs_src = target_specs
                .manifest_path
                .as_std_path()
                .parent()
                .and_then(|root| {
                    let src = root.join("target-specs");
                    src.is_dir().then_some(src)
                })
                .context("Could not find `target-specs` directory within `rustc_codegen_spirv-target-specs` dependency")?;
        log::info!(
            "target-specs: found `rustc_codegen_spirv-target-specs` with `target-specs` directory `{}`",
            target_specs_dst.display()
        );

        if source.is_path() {
            // skip copy
            log::info!(
                "target-specs resolution: source is local path, use target-specs directly from `{}`",
                target_specs_dst.display()
            );
            target_specs_dst = target_specs_src;
        } else {
            // copy over the target-specs
            log::info!(
                "target-specs resolution: coping target-specs from `{}`{}",
                target_specs_dst.display(),
                if update_files { "" } else { " was skipped" }
            );
            if update_files {
                copy_spec_files(&target_specs_src, &target_specs_dst)
                    .context("copying target-specs json files")?;
            }
        }
    } else {
        // use legacy target specs bundled with cargo gpu
        if source.is_path() {
            // This is a stupid situation:
            // * We can't be certain that there are `target-specs` in the local checkout (there may be some in `spirv-builder`)
            // * We can't dump our legacy ones into the `install_dir`, as that would modify the local rust-gpu checkout
            // -> do what the old cargo gpu did, one global dir for all target specs
            // and hope parallel runs don't shred each other
            target_specs_dst = cache_dir()?.join("legacy-target-specs-for-local-checkout");
        }
        log::info!(
            "target-specs resolution: legacy target specs in directory `{}`",
            target_specs_dst.display()
        );
        if update_files {
            log::info!(
                "target-specs: Writing legacy target specs into `{}`",
                target_specs_dst.display()
            );
            write_legacy_target_specs(&target_specs_dst)?;
        }
    }

    Ok(target_specs_dst)
}
