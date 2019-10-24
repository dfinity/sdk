#[macro_use]
extern crate lalrpop_util;
extern crate serde_idl;

lalrpop_mod!(pub idl);

#[test]
fn test() {
    use serde_idl::value::IDLValue;
    let args = idl::ArgsParser::new().parse("(true)").unwrap();
    assert_eq!(args, vec![IDLValue::Bool(true)]);
    let args = idl::ArgsParser::new().parse(" (true, null )").unwrap();
    assert_eq!(args, vec![IDLValue::Bool(true), IDLValue::Null]);
    let args = idl::ArgsParser::new()
        .parse("(true, null, 42, random, false)")
        .unwrap();
    assert_eq!(
        args,
        vec![
            IDLValue::Bool(true),
            IDLValue::Null,
            IDLValue::Int(42),
            IDLValue::Text("random".to_owned()),
            IDLValue::Bool(false)
        ]
    );
}
