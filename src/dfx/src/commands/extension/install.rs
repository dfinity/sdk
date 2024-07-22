use crate::commands::DfxCommand;
use crate::config::cache::DiskBasedCache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use anyhow::bail;
use clap::Parser;
use clap::Subcommand;
use dfx_core::extension::manager::InstallOutcome;
use dfx_core::extension::url::ExtensionJsonUrl;
use semver::Version;
use slog::{error, info, warn};
use tokio::runtime::Runtime;
use url::Url;

#[derive(Parser)]
pub struct InstallOpts {
    /// Specifies the name of the extension to install.
    name: String,
    /// Installs the extension under different name. Useful when installing an extension with the same name as: already installed extension, or a built-in command.
    #[clap(long)]
    install_as: Option<String>,
    /// Installs a specific version of the extension, bypassing version checks
    #[clap(long)]
    version: Option<Version>,
}

pub fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult<()> {
    // creating an `extensions` directory in an otherwise empty cache directory would
    // cause the cache to be considered "installed" and later commands would fail
    DiskBasedCache::install(&env.get_cache().version_str())?;
    let spinner = env.new_spinner(format!("Installing extension: {}", opts.name).into());
    let mgr = env.get_extension_manager();
    let effective_extension_name = opts.install_as.clone().unwrap_or_else(|| opts.name.clone());
    if DfxCommand::has_subcommand(&effective_extension_name) {
        bail!("Extension '{}' cannot be installed because it conflicts with an existing command. Consider using '--install-as' flag to install this extension under different name.", opts.name)
    }

    let url = if let Ok(url) = Url::parse(&opts.name) {
        ExtensionJsonUrl::new(url)
    } else {
        ExtensionJsonUrl::registered(&opts.name)?
    };

    let runtime = Runtime::new().expect("Unable to create a runtime");

    let install_outcome = runtime.block_on(async {
        mgr.install_extension(&url, opts.install_as.as_deref(), opts.version.as_ref())
            .await
    })?;
    spinner.finish_and_clear();
    let logger = env.get_logger();
    let install_as = if let Some(install_as) = opts.install_as {
        format!(", and is available as '{}'", install_as)
    } else {
        "".to_string()
    };
    let extension = &opts.name;
    match install_outcome {
        InstallOutcome::Installed(version) => {
            info!(logger, "Extension '{extension}' version {version} installed successfully{install_as}");
            Ok(())
        }
        InstallOutcome::AlreadyInstalled(version) => {
            warn!(logger, "Extension '{extension}' version {version} is already installed{install_as}"
            );
            match opts.version {
                Some(requested_version) if requested_version != version => {
                    error!(logger, "Requested version {requested_version} is different from installed version {version}",
                    );
                    error!(logger, r#"To upgrade, run "dfx extension uninstall {extension}" and then re-run the dfx extension install command"#);
                    bail!("Requested version not installed");
                }
                _ => Ok(()),
            }
        }
    }
}
