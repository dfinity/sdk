use std::io::stdin;

pub fn ask_for_consent(message: &str) -> Result<(), String> {
    eprintln!("WARNING!");
    eprintln!("{}", message);
    eprintln!("Do you want to proceed? yes/No");
    let mut input_string = String::new();
    stdin()
        .read_line(&mut input_string)
        .map_err(|_err| "Unable to read input".to_string())?;
    let input_string = input_string.trim_end();
    if input_string != "yes" {
        return Err("Refusing to install canister without approval".to_string());
    }
    Ok(())
}
