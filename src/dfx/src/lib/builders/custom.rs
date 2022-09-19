use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::custom::CustomCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;

use anyhow::{anyhow, bail, Context};
use bytes::Bytes;
use candid::Principal as CanisterId;
use console::style;
use fn_error_context::context;
use garcon::{Delay, Waiter};
use hyper_rustls::ConfigBuilderExt;
use reqwest::{Client, StatusCode};
use slog::info;
use slog::Logger;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use url::Url;

/// Set of extras that can be specified in the dfx.json.
struct CustomBuilderExtra {
    /// A list of canister names to use as dependencies.
    dependencies: Vec<CanisterId>,
    /// Where to download the wasm from
    input_wasm_url: Option<Url>,
    /// Where the wasm output will be located.
    wasm: PathBuf,
    /// Where to download the candid from
    input_candid_url: Option<Url>,
    /// Where the candid output will be located.
    candid: PathBuf,
    /// A command to run to build this canister. This is optional if the canister
    /// only needs to exist.
    build: Vec<String>,
}

impl CustomBuilderExtra {
    #[context("Failed to create CustomBuilderExtra for canister '{}'.", info.get_name())]
    fn try_from(info: &CanisterInfo, pool: &CanisterPool) -> DfxResult<Self> {
        let dependencies = info.get_dependencies()
            .iter()
            .map(|name| {
                pool.get_first_canister_with_name(name)
                    .map(|c| c.canister_id())
                    .map_or_else(
                        || Err(anyhow!("A canister with the name '{}' was not found in the current project.", name.clone())),
                        DfxResult::Ok,
                    )
            })
            .collect::<DfxResult<Vec<CanisterId>>>().with_context( || format!("Failed to collect dependencies (canister ids) of canister {}.", info.get_name()))?;
        let info = info.as_info::<CustomCanisterInfo>()?;
        let input_wasm_url = info.get_input_wasm_url().to_owned();
        let wasm = info.get_output_wasm_path().to_owned();
        let input_candid_url = info.get_input_candid_url().to_owned();
        let candid = info.get_output_idl_path().to_owned();
        let build = info.get_build_tasks().to_owned();

        Ok(CustomBuilderExtra {
            dependencies,
            input_wasm_url,
            wasm,
            input_candid_url,
            candid,
            build,
        })
    }
}

/// A Builder for a WASM type canister, which has an optional build step.
/// This will set environment variables for the external tool;
///   `CANISTER_ID`     => Its own canister ID (in textual format).
///   `CANDID_PATH`     => Its own candid path.
///   `CANISTER_ID_{}`  => The canister ID of all dependencies. `{}` is replaced by the name.
///   `CANDID_{}`       => The candid path of all dependencies. `{}` is replaced by the name.
pub struct CustomBuilder {
    logger: Logger,
}

impl CustomBuilder {
    #[context("Failed to create CustomBuilder.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(CustomBuilder {
            logger: env.get_logger().clone(),
        })
    }
}

impl CanisterBuilder for CustomBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        Ok(CustomBuilderExtra::try_from(info, pool)?.dependencies)
    }

    #[context("Failed to build custom canister {}.", info.get_name())]
    fn build(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let CustomBuilderExtra {
            input_candid_url: _,
            candid,
            input_wasm_url: _,
            wasm,
            build,
            dependencies,
        } = CustomBuilderExtra::try_from(info, pool)?;

        let canister_id = info.get_canister_id().unwrap();
        let vars = super::environment_variables(info, &config.network_name, pool, &dependencies);

        let mut add_candid_service_metadata = false;
        for command in build {
            info!(
                self.logger,
                r#"{} '{}'"#,
                style("Executing").green().bold(),
                command
            );

            // First separate everything as if it was read from a shell.
            let args = shell_words::split(&command)
                .with_context(|| format!("Cannot parse command '{}'.", command))?;
            // No commands, noop.
            if !args.is_empty() {
                add_candid_service_metadata = true;
                run_command(args, &vars, info.get_workspace_root())
                    .with_context(|| format!("Failed to run {}.", command))?;
            }
        }

        let mut file =
            File::open(&wasm).with_context(|| format!("Failed to open {}", wasm.display()))?;
        let mut header = [0; 4];
        file.read_exact(&mut header)?;
        if header != *b"\0asm" {
            add_candid_service_metadata = false;
        }

        // Custom canister may have WASM gzipped
        if info.get_shrink() && header == *b"\0asm" {
            info!(self.logger, "Shrink WASM module size.");
            super::shrink_wasm(&wasm)?;
        }

        Ok(BuildOutput {
            canister_id,
            wasm: WasmBuildOutput::File(wasm),
            idl: IdlBuildOutput::File(candid),
            add_candid_service_metadata,
        })
    }

    fn generate_idl(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        let generate_output_dir = &info
            .get_declarations_config()
            .output
            .as_ref()
            .context("output here must not be None")?;

        std::fs::create_dir_all(generate_output_dir).with_context(|| {
            format!(
                "Failed to create {}.",
                generate_output_dir.to_string_lossy()
            )
        })?;

        let output_idl_path = generate_output_dir
            .join(info.get_name())
            .with_extension("did");

        // get the path to candid file
        let CustomBuilderExtra { candid, .. } = CustomBuilderExtra::try_from(info, pool)?;

        std::fs::copy(&candid, &output_idl_path).with_context(|| {
            format!(
                "Failed to copy canidid from {} to {}.",
                candid.to_string_lossy(),
                output_idl_path.to_string_lossy()
            )
        })?;

        Ok(output_idl_path)
    }
}

fn run_command(args: Vec<String>, vars: &[super::Env<'_>], cwd: &Path) -> DfxResult<()> {
    let (command_name, arguments) = args.split_first().unwrap();
    let canonicalized = cwd
        .join(command_name)
        .canonicalize()
        .or_else(|_| which::which(command_name))
        .map_err(|_| anyhow!("Cannot find command or file {command_name}"))?;
    let mut cmd = Command::new(&canonicalized);

    cmd.args(arguments)
        .current_dir(cwd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (key, value) in vars {
        cmd.env(key.as_ref(), value);
    }

    let output = cmd
        .output()
        .with_context(|| format!("Error executing custom build step {cmd:#?}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(DfxError::new(BuildError::CustomToolError(
            output.status.code(),
        )))
    }
}

pub async fn custom_download(info: &CanisterInfo, pool: &CanisterPool) -> DfxResult {
    let CustomBuilderExtra {
        input_candid_url,
        candid,
        input_wasm_url,
        wasm,
        build: _,
        dependencies: _,
    } = CustomBuilderExtra::try_from(info, pool)?;

    if let Some(url) = input_wasm_url {
        download_file(&url, &wasm).await?;
    }
    if let Some(url) = input_candid_url {
        download_file(&url, &candid).await?;
    }

    Ok(())
}

#[context("Failed to download {} to {}.", from, to.display())]
async fn download_file(from: &Url, to: &Path) -> DfxResult {
    let parent_dir = to.parent().unwrap();
    create_dir_all(&parent_dir).with_context(|| {
        format!(
            "Failed to create output directory {}.",
            parent_dir.display()
        )
    })?;

    let tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_webpki_roots()
        .with_no_client_auth();

    let client = reqwest::Client::builder()
        .use_preconfigured_tls(tls_config)
        .build()
        .context("Could not create HTTP client.")?;

    let mut waiter = Delay::builder()
        .throttle(Duration::from_millis(1000))
        .with(Delay::count_timeout(5))
        .exponential_backoff_capped(Duration::from_millis(500), 1.4, Duration::from_secs(5))
        .build();
    waiter.start();

    let body = loop {
        match attempt_download(&client, from).await {
            Ok(Some(body)) => break Ok(body),
            Ok(None) => bail!("Not found: {}", from),
            Err(request_error) => {
                if let Err(_waiter_err) = waiter.async_wait().await {
                    break Err(request_error);
                }
            }
        }
    }?;

    fs::write(to, body).with_context(|| format!("Failed to write {}", to.display()))?;

    Ok(())
}

async fn attempt_download(client: &Client, url: &Url) -> DfxResult<Option<Bytes>> {
    let response = client.get(url.clone()).send().await?;

    if response.status() == StatusCode::NOT_FOUND {
        Ok(None)
    } else {
        let body = response.error_for_status()?.bytes().await?;
        Ok(Some(body))
    }
}
