#![no_main]

#[ic_cdk::query]
fn name() -> String {
    "bin".into()
}
