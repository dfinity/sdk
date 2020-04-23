use ic_agent::Principal;

// Note perhaps in the future we will need to indicate the schema
// type.
#[derive(Clone)]
pub struct Signature {
    pub signer: Principal,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}
