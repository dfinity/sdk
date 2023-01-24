use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

use std::io::Write;

#[derive(Parser, Debug)]
pub struct ListOpts {
    #[clap(use_value_delimiter = true, value_delimiter = ',')]
    /// Comma-separated list of fields from extension's metadata.json to display.
    metadata: Vec<String>,
}

pub fn exec(env: &dyn Environment, opts: ListOpts) -> DfxResult<()> {
    let mgr = env.new_extension_manager()?;
    let extensions = mgr.list_installed_extensions()?;

    if extensions.is_empty() {
        eprintln!("No extensions installed.");
        return Ok(());
    }

    if opts.metadata.is_empty() {
        eprintln!("Installed extensions:");
        for extension in extensions {
            eprint!("  ");
            std::io::stderr().flush()?;
            println!("{}", extension);
            std::io::stdout().flush()?;
        }
    } else {
        // TODO diplay only the metadata specified in opts.metadata
        println!("{:#?}", opts.metadata); // TODO delete this LoC
        for extension in extensions {
            let ext_manifest = mgr.load_manifest(extension)?;
            println!("{}", ext_manifest); // TODO: print nice table
        }
    }
    Ok(())
}
