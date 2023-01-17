use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::extension::manager::ExtensionsManager;

pub fn exec(env: &dyn Environment) -> DfxResult<()> {
    let mgr = ExtensionsManager::new(env)?;
    let extensions = mgr.list_installed_extensions();
    if extensions.is_empty() {
        println!("No extensions installed.");
    } else {
        println!("Installed extensions:");
        for extension in extensions {
            println!("  {}", extension);
        }
    }
    Ok(())
}
