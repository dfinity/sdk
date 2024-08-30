use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;
use std::io::Write;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Parser)]
pub struct ListOpts {
    /// Specifies to list the available remote extensions.
    #[arg(long)]
    available: bool,
    /// Specifies the URL of the catalog to use to find the extension.
    #[clap(long)]
    catalog_url: Option<Url>,
}

pub fn exec(env: &dyn Environment, opts: ListOpts) -> DfxResult<()> {
    let mgr = env.get_extension_manager();
    let extensions;

    let result;
    if opts.available || opts.catalog_url.is_some() {
        let runtime = Runtime::new().expect("Unable to create a runtime");
        extensions = runtime.block_on(async {
            mgr.list_available_extensions(opts.catalog_url.as_ref())
                .await
        })?;

        result = display_extension_list(
            &extensions,
            "No extensions available.",
            "Available extensions:",
        );
    } else {
        extensions = mgr.list_installed_extensions()?;

        result = display_extension_list(
            &extensions,
            "No extensions installed.",
            "Installed extensions:",
        );
    };

    result
}

fn display_extension_list(
    extensions: &Vec<String>,
    empty_msg: &str,
    header_msg: &str,
) -> DfxResult<()> {
    if extensions.is_empty() {
        eprintln!("{}", empty_msg);
        return Ok(());
    }

    eprintln!("{}", header_msg);
    for extension in extensions {
        eprint!("  ");
        std::io::stderr().flush()?;
        println!("{}", extension);
        std::io::stdout().flush()?;
    }

    Ok(())
}
