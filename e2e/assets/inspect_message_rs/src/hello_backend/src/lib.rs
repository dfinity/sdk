use ic_cdk::{
    api::{
        call,
    },
};

#[ic_cdk::update]
fn always_accepted() {
}

#[ic_cdk::update]
fn always_rejected() {
}

#[ic_cdk::inspect_message]
fn inspect_message() {
  if call::method_name().as_str() == "always_accepted"{
    call::accept_message();
  }
}
