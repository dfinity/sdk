use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use std::io::Write;

pub fn exec(env: &dyn Environment) -> DfxResult<()> {
    let mgr = env.new_extension_manager()?;
    let extensions = mgr.list_installed_extensions()?;

    if extensions.is_empty() {
        eprintln!("No extensions installed.");
        return Ok(());
    }

    eprintln!("Installed extensions:");
    for extension in extensions {
        eprint!("  ");
        std::io::stderr().flush()?;
        println!("{}", extension);
        std::io::stdout().flush()?;
    }
    Ok(())
}
