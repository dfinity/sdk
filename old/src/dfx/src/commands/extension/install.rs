use crate::commands::DfxCommand;
use crate::config::cache::DiskBasedCache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use anyhow::bail;
use clap::Parser;
use clap::Subcommand;
use semver::Version;

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

    mgr.install_extension(
        &opts.name,
        opts.install_as.as_deref(),
        opts.version.as_ref(),
    )?;
    spinner.finish_with_message(
        format!(
            "Extension '{}' installed successfully{}",
            opts.name,
            if let Some(install_as) = opts.install_as {
                format!(", and is available as '{}'", install_as)
            } else {
                "".to_string()
            }
        )
        .into(),
    );
    Ok(())
}
