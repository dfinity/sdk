extern crate serde_idl;

use serde_idl::grammar::IDLProgParser;
use serde_idl::lexer::Lexer;
use serde_idl::types::*;
use serde_idl::value::ParserError;

fn parse_idl(input: &str) -> IDLProg {
    let lexer = Lexer::new(input);
    IDLProgParser::new().parse(lexer).unwrap()
}

fn parse_idl_err(input: &str) -> Result<IDLProg, ParserError> {
    let lexer = Lexer::new(input);
    IDLProgParser::new().parse(lexer)
}

#[test]
fn parse_idl_prog() {
    let prog = r#"
type my_type = nat;
service server {
  f : (nat) -> ();
  g : (my_type) -> (int) query;
}
    "#;
    let ast = parse_idl(&prog);
    assert_eq!(format!("{}", ast), "", "\n\n{}\n", ast);
}
