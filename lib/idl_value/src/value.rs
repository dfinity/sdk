#[derive(Debug, PartialEq)]
pub enum IDLValue {
    Bool(bool),
    Null,
    Text(String),
    Int(i64),
    Nat(u64),
    Opt(Box<IDLValue>),
    Vec(Vec<IDLValue>),
}
