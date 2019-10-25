extern crate serde_idl;

use serde_idl::value::{IDLField, IDLValue};
use serde_idl::ArgsParser;

#[test]
fn parse() {
    let args = ArgsParser::new().parse("(true)").unwrap();
    assert_eq!(args, vec![IDLValue::Bool(true)]);
    let args = ArgsParser::new().parse(" (true, null )").unwrap();
    assert_eq!(args, vec![IDLValue::Bool(true), IDLValue::Null]);
    let args = ArgsParser::new()
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
    let args = ArgsParser::new().parse("(vec{1;2;3;4})").unwrap();
    assert_eq!(
        args,
        vec![IDLValue::Vec(vec![
            IDLValue::Int(1),
            IDLValue::Int(2),
            IDLValue::Int(3),
            IDLValue::Int(4)
        ])]
    );
    let args = ArgsParser::new()
        .parse("(opt record {}, record { 1=42;44=test; 2=false }, variant { 5=null })")
        .unwrap();
    assert_eq!(
        args,
        vec![
            IDLValue::Opt(Box::new(IDLValue::Record(vec![]))),
            IDLValue::Record(vec![
                IDLField {
                    id: 1,
                    val: IDLValue::Int(42)
                },
                IDLField {
                    id: 2,
                    val: IDLValue::Bool(false)
                },
                IDLField {
                    id: 44,
                    val: IDLValue::Text("test".to_owned())
                },
            ]),
            IDLValue::Variant(Box::new(IDLField {
                id: 5,
                val: IDLValue::Null
            }))
        ]
    );
}
