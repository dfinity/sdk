use crate::lib::error::DfxResult;
use crate::CliOpts;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use clap_complete::Shell;

/// Generate a shell autocompletion script.
#[derive(Parser)]
pub struct AutocompleteOpts {
    /// The shell for which to generate autocomplete scripts
    #[clap(long, default_value("bash"))]
    shell: Shell,

    /// The name of the binary
    #[clap(long, default_value("dfx"))]
    bin_name: String,

    #[clap(name = "OUTPUT_FILE", parse(from_os_str), default_value("-"))]
    output_file: PathBuf,
}

pub fn exec(opts: AutocompleteOpts) -> DfxResult {
    let mut output: Box<dyn Write> =
        if opts.output_file == PathBuf::from("-") {
            Box::new(std::io::stdout())
        } else {
            Box::new(File::create(&opts.output_file).with_context(|| {
                format!("Unable to open {} for output", opts.output_file.display())
            })?)
        };
    generate(
        opts.shell,
        &mut CliOpts::command(),
        opts.bin_name,
        &mut *output,
    );
    Ok(())
}
