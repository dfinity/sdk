use std::io::stdin;

use crate::error::cli::UserConsent;

pub fn ask_for_consent(message: &str) -> Result<(), UserConsent> {
    eprintln!("WARNING!");
    eprintln!("{}", message);
    eprintln!("Do you want to proceed? yes/No");
    let mut input_string = String::new();
    stdin()
        .read_line(&mut input_string)
        .map_err(|err| UserConsent::ReadError(err))?;
    let input_string = input_string.trim_end();
    if input_string != "yes" {
        return Err(UserConsent::Declined);
    }
    Ok(())
}
