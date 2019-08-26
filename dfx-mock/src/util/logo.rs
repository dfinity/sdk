pub fn generate_logo() -> String {
    // Move the logo to the right by 8 characters.
    format!(
        "        {}",
        include_str!("../../assets/dfinity.aart").to_string()
            .replace("\n", "\n        "))
}
