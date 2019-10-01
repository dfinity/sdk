#[macro_use]
extern crate serde_idl;
extern crate serde;
extern crate dfx_info;

//use dfx_info::types::{get_type, Type};
use dfx_info::IDLType;
//use serde::Deserialize;
use serde_idl::{from_bytes, idl_hash};

fn test_check<T>(x:&T)
where T: Eq + IDLType + serde::de::DeserializeOwned + std::fmt::Debug,
{
    // serialize via the IDL's wire format:
    let y = IDL!(x);
    // deserialize from the IDL's wire format:
    let z1 : Result<T, _> = from_bytes(&y);
    let z2 : T = z1.unwrap();
    assert_eq!(x, &z2);
}

#[test]
fn test_simple_values() {
    test_check(&true);
    test_check(&false);
    test_check(&(true, true));
    test_check(&(1, true));
    test_check(&(false, 2));
    test_check(&(false, 2, 3));
}
