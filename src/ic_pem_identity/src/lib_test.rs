#![cfg(test)]
use crate::PemIdentity;
use ic_http_agent::{
    to_request_id, Blob, CanisterId, MessageWithSender, ReadRequest, Request, Signer,
};
use ring::signature;

#[test]
fn works() {
    let temp_root = tempfile::tempdir().unwrap();
    let pem_file_path = temp_root.into_path().join("test.pem");

    // Generate a key pair.
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

    let pem = pem::Pem {
        tag: "PRIVATE KEY".to_owned(),
        contents: Vec::from(pkcs8_bytes.as_ref()),
    };

    std::fs::write(&pem_file_path, pem::encode(&pem)).unwrap();

    let signer = PemIdentity::new(&pem_file_path).unwrap();

    let arg = Blob::empty();
    let canister_id = CanisterId::from(Blob::empty());
    let request = Request::Read(ReadRequest::Query {
        arg: &arg,
        canister_id: &canister_id,
        method_name: "Some Method",
    });

    let request_with_sender = MessageWithSender {
        request: request.clone(),
        sender: signer.principal_id.clone(),
    };
    let actual_request_id =
        to_request_id(&request_with_sender).expect("Failed to produce request id");
    let request_id = signer.sign(request).expect("Failed to sign").0;
    assert_eq!(request_id, actual_request_id)
}
