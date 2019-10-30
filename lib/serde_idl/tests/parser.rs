extern crate serde_idl;

use serde_idl::grammar::ArgsParser;
use serde_idl::lexer::Lexer;
use serde_idl::value::{IDLArgs, IDLField, IDLValue, ParserError};

fn parse_args(input: &str) -> IDLArgs {
    let lexer = Lexer::new(input);
    ArgsParser::new().parse(lexer).unwrap()
}

fn parse_args_err(input: &str) -> Result<IDLArgs, ParserError<'_>> {
    let lexer = Lexer::new(input);
    ArgsParser::new().parse(lexer)
}

#[test]
fn parse_bool_lit() {
    let args = parse_args("(true)");
    assert_eq!(args.args, vec![IDLValue::Bool(true)]);
    assert_eq!(format!("{}", args), "(true)");
}

#[test]
fn parse_literals() {
    let args = parse_args(" (true, null )");
    assert_eq!(args.args, vec![IDLValue::Bool(true), IDLValue::Null]);
    assert_eq!(format!("{}", args), "(true, null)");
}

#[test]
fn parse_more_literals() {
    let args = parse_args("(true, null, 42, \"random\", \"string with whitespace\", +42, -42, false)");
    assert_eq!(
        args.args,
        vec![
            IDLValue::Bool(true),
            IDLValue::Null,
            IDLValue::Int(42),
            IDLValue::Text("random".to_owned()),
            IDLValue::Text("string with whitespace".to_owned()),
            IDLValue::Int(42),
            IDLValue::Int(-42),
            IDLValue::Bool(false)
        ]
    );
    assert_eq!(format!("{}", args),
               "(true, null, 42, \"random\", \"string with whitespace\", 42, -42, false)");
}

#[test]
fn parse_vec() {
    let args = parse_args("(vec{1;2;3;4})");
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
}

#[test]
fn parse_optional_record() {
    let args =
        parse_args("(opt record {}, record { 1=42;44=\"test\"; 2=false }, variant { 5=null })");
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
        "(opt record { }, record { 1 = 42; 2 = false; 44 = \"test\"; }, variant { 5 = null })"
    );
}

#[test]
fn parse_nested_record() {
    let args = parse_args(
        "(record {label=42; 43=record {test=\"test\"; msg=\"hello\"}; long_label=opt null})",
    );
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
    assert_eq!(format!("{}", args), "(record { 43 = record { 5446209 = \"hello\"; 1291438162 = \"test\"; }; 1350385585 = opt null; 1873743348 = 42; })");
}

#[test]
fn parse_escape_sequence() {
    let result = parse_args("(\"\\n\")");
    assert_eq!(format!("{}", result), "(\"\\n\")")
}

#[test]
fn parse_illegal_escape_sequence() {
    let result = parse_args_err("(\"\\q\")");
    assert_eq!(format!("{}", result.unwrap_err()), "Unknown escape \\q")
}
