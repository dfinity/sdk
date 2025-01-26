#[ic_cdk::query]
fn greet(s: String) -> String {
    format!("Hello, {s}!")
}

#[ic_cdk::update]
fn greet_update(s: String) -> String {
    format!("Hello, {s}!")
}
