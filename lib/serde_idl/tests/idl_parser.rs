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
type f = func (List, func (int) -> (int)) -> (opt List);
type broker = service {
  find : (text) ->
    (service {up:() -> (); current:() -> (nat)});
};

service server {
  f : (nat, opt bool) -> () oneway;
  g : (my_type, List, opt List) -> (int) query;
  h : (vec opt text, variant { A: nat; B: opt text }, opt List) -> (record { id: nat; 0x2a: record {} });
  i : f;
}
    "#;
    let pretty_80 = r#"import "test.did";
type my_type = nat;
type List = record { head: int; tail: List; };
type f = func (List, func (int) -> (int)) -> (opt List);
type broker = service {
      find: (text) -> (service {
             up: () -> ();
             current: () -> (nat); }); };
service server {
  f: (nat, opt bool) -> () oneway;
  g: (my_type, List, opt List) -> (int) query;
  h:
    (vec opt text, variant { A: nat; B: opt text; }, opt List)
    -> (record { id: nat; 42: record { }; });
  i: f;
}"#;
    let ast = parse_idl(&prog);
    assert_eq!(ast.to_pretty(80), pretty_80);
    let ast2 = parse_idl(&pretty_80);
    assert_eq!(ast2.to_pretty(80), pretty_80);
}
