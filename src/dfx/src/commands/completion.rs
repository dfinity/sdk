use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::CliOpts;
use anyhow::Context;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use clap_complete::Shell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Generate a shell completion script.
#[derive(Parser)]
pub struct CompletionOpts {
    /// The shell for which to generate completion scripts
    #[clap(long, default_value("bash"))]
    shell: Shell,

    /// The name of the binary
    #[clap(long, default_value("dfx"))]
    bin_name: String,

    /// The file to write the completion script to, or "-" for stdout (the default).
    #[clap(name = "OUTPUT_FILE", default_value("-"))]
    output_file: PathBuf,
}

pub fn exec(env: &dyn Environment, opts: CompletionOpts) -> DfxResult {
    let em = env.get_extension_manager();
    let mut output: Box<dyn Write> =
        if opts.output_file == PathBuf::from("-") {
            Box::new(std::io::stdout())
        } else {
            Box::new(File::create(&opts.output_file).with_context(|| {
                format!("Unable to open {} for output", opts.output_file.display())
            })?)
        };
    let installed_extensions = em.installed_extensions_as_clap_commands()?;
    let mut command = if installed_extensions.is_empty() {
        CliOpts::command()
    } else {
        CliOpts::command_for_update().subcommands(&installed_extensions)
    };

    generate(opts.shell, &mut command, opts.bin_name, &mut *output);
    Ok(())
}
