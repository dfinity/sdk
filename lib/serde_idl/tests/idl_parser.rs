extern crate serde_idl;

use serde_idl::grammar::IDLProgParser;
use serde_idl::lexer::Lexer;
use serde_idl::types::IDLProg;

fn parse_idl(input: &str) -> IDLProg {
    let lexer = Lexer::new(input);
    IDLProgParser::new().parse(lexer).unwrap()
}

#[test]
fn parse_idl_prog() {
    let prog = r#"
import "test.did";
type my_type = nat;
type List = record { head: int; tail: List };
type broker = service {
  find : (text) ->
    (service {up:() -> (); current:() -> (nat)});
};

service server {
  f : (nat, opt bool) -> () oneway;
  g : (my_type, List, opt List) -> (int) query;
  h : (vec opt text, variant { A: nat; B: opt text }) -> (record { id: nat; 0x2a: record {} });
}
    "#;
    let ast = parse_idl(&prog);
    let output = ast.to_pretty(80);
    assert_eq!(output, "", "\n\n{}\n", output);
}
