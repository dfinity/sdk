extern crate serde_idl;

use serde_idl::value::{IDLField, IDLValue};
use serde_idl::ArgsParser;

#[test]
fn parse() {
    let args = ArgsParser::new().parse("(true)").unwrap();
    assert_eq!(args.args, vec![IDLValue::Bool(true)]);
    assert_eq!(format!("{}", args), "(true)");

    let args = ArgsParser::new().parse(" (true, null )").unwrap();
    assert_eq!(args.args, vec![IDLValue::Bool(true), IDLValue::Null]);
    assert_eq!(format!("{}", args), "(true, null)");

    let args = ArgsParser::new()
        .parse("(true, null, 42, random, false)")
        .unwrap();
    assert_eq!(
        args.args,
        vec![
            IDLValue::Bool(true),
            IDLValue::Null,
            IDLValue::Int(42),
            IDLValue::Text("random".to_owned()),
            IDLValue::Bool(false)
        ]
    );
    assert_eq!(format!("{}", args), "(true, null, 42, random, false)");

    let args = ArgsParser::new().parse("(vec{1;2;3;4})").unwrap();
    assert_eq!(
        args.args,
        vec![IDLValue::Vec(vec![
            IDLValue::Int(1),
            IDLValue::Int(2),
            IDLValue::Int(3),
            IDLValue::Int(4)
        ])]
    );
    assert_eq!(format!("{}", args), "(vec { 1; 2; 3; 4; })");

    let args = ArgsParser::new()
        .parse("(opt record {}, record { 1=42;44=test; 2=false }, variant { 5=null })")
        .unwrap();
    assert_eq!(
        args.args,
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
    assert_eq!(
        format!("{}", args),
        "(opt record { }, record { 1 = 42; 2 = false; 44 = test; }, variant { 5 = null })"
    );

    let args = ArgsParser::new()
        .parse("(record {label=42; 43=record {test=test; msg=hello}; long_label=opt null})")
        .unwrap();
    assert_eq!(
        args.args,
        vec![IDLValue::Record(vec![
            IDLField {
                id: 43,
                val: IDLValue::Record(vec![
                    IDLField {
                        id: 5_446_209,
                        val: IDLValue::Text("hello".to_owned())
                    },
                    IDLField {
                        id: 1_291_438_162,
                        val: IDLValue::Text("test".to_owned())
                    }
                ])
            },
            IDLField {
                id: 1_350_385_585,
                val: IDLValue::Opt(Box::new(IDLValue::Null))
            },
            IDLField {
                id: 1_873_743_348,
                val: IDLValue::Int(42)
            }
        ])]
    );
    assert_eq!(format!("{}", args), "(record { 43 = record { 5446209 = hello; 1291438162 = test; }; 1350385585 = opt null; 1873743348 = 42; })");
}
