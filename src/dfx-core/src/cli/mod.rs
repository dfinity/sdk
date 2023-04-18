use crate::error::cli::UserConsent;

use std::io::stdin;

pub fn ask_for_consent(message: &str) -> Result<(), UserConsent> {
    eprintln!("WARNING!");
    eprintln!("{}", message);
    eprintln!("Do you want to proceed? yes/No");
    let mut input_string = String::new();
    stdin()
        .read_line(&mut input_string)
        .map_err(UserConsent::ReadError)?;
    let input_string = input_string.trim_end().to_lowercase();
    if input_string != "yes" && input_string != "y" {
        return Err(UserConsent::Declined);
    }
    Ok(())
}
