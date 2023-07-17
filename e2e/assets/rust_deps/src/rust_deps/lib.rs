use ic_cdk::update;

mod declarations;
use declarations::multiply_deps::multiply_deps;

// Inter-canister call can only be from a update call
#[update]
async fn read() -> candid::Nat {
    multiply_deps.read().await.unwrap().0
}
