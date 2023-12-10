use std::time::Duration;

pub fn network_to_pathcompat(network_name: &str) -> String {
    network_name.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}

pub fn expiry_duration() -> Duration {
    // 5 minutes is max ingress timeout
    // 4 minutes accounts for possible replica drift
    Duration::from_secs(60 * 4)
}
