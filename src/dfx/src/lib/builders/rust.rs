use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::rust::RustCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use anyhow::{bail, Context};
use candid::Principal as CanisterId;
use fn_error_context::context;
use slog::{info, o};
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

pub struct RustBuilder {
    logger: slog::Logger,
}

impl RustBuilder {
    #[context("Failed to create RustBuilder.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(RustBuilder {
            logger: env.get_logger().new(o! {
                "module" => "rust"
            }),
        })
    }
}

impl CanisterBuilder for RustBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        super::collect_dependencies(info, pool)
    }

    #[context("Failed to build Rust canister '{}'.", canister_info.get_name())]
    fn build(
        &self,
        pool: &CanisterPool,
        canister_info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let rust_info = canister_info.as_info::<RustCanisterInfo>()?;
        let package = rust_info.get_package();

        let canister_id = canister_info.get_canister_id().unwrap();

        let mut cargo = Command::new("cargo");
        cargo
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--release")
            .arg("-p")
            .arg(package)
            .arg("--locked");

        let dependencies = self
            .get_dependencies(pool, canister_info)
            .unwrap_or_default();
        let vars = super::get_and_write_environment_variables(
            canister_info,
            &config.network_name,
            pool,
            &dependencies,
            config.env_file.as_deref(),
        )?;
        for (key, val) in vars {
            cargo.env(key.as_ref(), val);
        }

        info!(
            self.logger,
            "Executing: cargo build --target wasm32-unknown-unknown --release -p {} --locked",
            package
        );
        let output = cargo.output().context("Failed to run 'cargo build'. You might need to run `cargo update` (or a similar command like `cargo vendor`) if you have updated `Cargo.toml`, because `dfx build` uses the --locked flag with Cargo.")?;

        if !output.status.success() {
            bail!("Failed to compile the rust package: {}", package);
        }

        Ok(BuildOutput {
            canister_id,
            wasm: WasmBuildOutput::File(rust_info.get_output_wasm_path().to_path_buf()),
            idl: IdlBuildOutput::File(rust_info.get_output_idl_path().to_path_buf()),
        })
    }

    fn generate_idl(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        let rust_info = info.as_info::<RustCanisterInfo>()?;
        let output_idl_path = rust_info.get_output_idl_path();
        if output_idl_path.exists() {
            Ok(output_idl_path.to_path_buf())
        } else {
            bail!(
                "Candid file: {} doesn't exist.",
                output_idl_path.to_string_lossy()
            );
        }
    }
}
