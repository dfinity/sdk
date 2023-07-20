use crate::commands::DfxCommand;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use dfx_core::cli::ask_for_consent;
use dfx_core::error::extension::ExtensionError;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct InstallOpts {
    /// Specifies the name of the extension to install.
    name: String,

    /// Use external (non-DFINITY) registry to install the extension.
    #[clap(long)]
    registry: Option<String>,

    /// Installs the extension under different name. Useful when installing an extension with the same name as: already installed extension, or a built-in command.
    #[clap(long)]
    install_as: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult<()> {
    if opts.registry.is_some() {
        ask_for_consent("DFINITY cannot guarantee the authenticity of extensions installed from external registries. You're installing it at your own risk!")?;
    }
    let spinner = env.new_spinner(format!("Installing extension: {}", opts.name).into());
    let mgr = env.new_extension_manager()?;
    let effective_extension_name = opts.install_as.clone().unwrap_or_else(|| opts.name.clone());
    if DfxCommand::has_subcommand(&effective_extension_name) {
        return Err(ExtensionError::CommandAlreadyExists(effective_extension_name).into());
    }

    mgr.install_extension(
        &opts.name,
        opts.registry.as_deref(),
        opts.install_as.as_deref(),
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
