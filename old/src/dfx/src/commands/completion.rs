use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::CliOpts;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use clap_complete::Shell;

/// Generate a shell completion script.
#[derive(Parser)]
pub struct CompletionOpts {
    /// The name of the command. Only needed if referring to dfx by another name, such as with an alias.
    #[clap(long, default_value("dfx"))]
    bin_name: String,

    /// The shell for which to generate completion scripts
    #[clap(default_value("bash"))]
    shell: Shell,
}

pub fn exec(env: &dyn Environment, opts: CompletionOpts) -> DfxResult {
    let em = env.get_extension_manager();
    let installed_extensions = em.installed_extensions_as_clap_commands()?;
    let mut command = if installed_extensions.is_empty() {
        CliOpts::command()
    } else {
        CliOpts::command_for_update().subcommands(&installed_extensions)
    };

    generate(
        opts.shell,
        &mut command,
        opts.bin_name,
        &mut std::io::stdout(),
    );
    Ok(())
}
