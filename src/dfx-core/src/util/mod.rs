pub fn network_to_pathcompat(network_name: &str) -> String {
    network_name.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}
