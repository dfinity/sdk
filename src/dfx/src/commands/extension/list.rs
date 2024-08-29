use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;
use std::io::Write;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Parser)]
pub struct ListOpts {
    /// Specifies to list the installed extensions.
    #[arg(long, conflicts_with("catalog_url"))]
    installed: bool,
    /// Specifies the URL of the catalog to use to find the extension.
    #[clap(long)]
    catalog_url: Option<Url>,
}

pub fn exec(env: &dyn Environment, opts: ListOpts) -> DfxResult<()> {
    let mgr = env.get_extension_manager();
    let extensions;
    let extension_msg_1;
    let extension_msg_2;

    if opts.installed {
        extensions = mgr.list_installed_extensions()?;
        extension_msg_1 = "No extensions installed.";
        extension_msg_2 = "Installed extensions:";
    } else {
        let runtime = Runtime::new().expect("Unable to create a runtime");
        extensions = runtime
            .block_on(async { mgr.list_remote_extensions(opts.catalog_url.as_ref()).await })?;

        extension_msg_1 = "No remote extensions available.";
        extension_msg_2 = "Remote extensions:";
    };

    if extensions.is_empty() {
        eprintln!("{}", extension_msg_1);
        return Ok(());
    }

    eprintln!("{}", extension_msg_2);
    for extension in extensions {
        eprint!("  ");
        std::io::stderr().flush()?;
        println!("{}", extension);
        std::io::stdout().flush()?;
    }

    Ok(())
}
