use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

pub fn exec(
    env: &dyn Environment,
) -> DfxResult<()> {
    let x = env
        .get_cache()
        .get_extensions_directory()
        .unwrap()
        .read_dir()?;

    let mut counter = 0;
    let mut names = Vec::new();

    for entry in x {
        counter += 1;
        let entry = entry.unwrap();
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        names.push(format!("  {}",name));
    }
    if counter == 0 {
        println!("No extensions installed");
    } else {
        println!("Installed extensions:");
        for name in names {
            println!("{}", name);
        }
    }
    Ok(())
}
