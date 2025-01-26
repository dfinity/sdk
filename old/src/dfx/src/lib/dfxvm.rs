use console::Style;

pub fn display_dfxvm_installation_instructions() {
    println!("You can install dfxvm by running the following command:");
    println!();
    let command = Style::new()
        .cyan()
        .apply_to(r#"sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)""#);
    println!("    {command}");
}
