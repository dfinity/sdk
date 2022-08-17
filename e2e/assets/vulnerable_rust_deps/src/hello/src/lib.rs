use std::time::Duration;

#[ic_cdk_macros::query]
fn greet(name: String) -> String {
    let _dur = Duration::from_secs(1);
    format!("Hello, {}!", name)
}
