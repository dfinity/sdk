extern crate pretty;
use self::pretty::{BoxDoc, Doc};
use std::fmt;

#[derive(Debug)]
pub enum IDLType {
    PrimT(PrimType),
    VarT(String),
    FuncT(FuncType),
    OptT(Box<IDLType>),
    VecT(Box<IDLType>),
    RecordT(Vec<TypeField>),
    VariantT(Vec<TypeField>),
    ServT(Vec<Binding>),
}

#[derive(Debug)]
pub enum PrimType {
    Nat,
    Int,
    Bool,
    Text,
    Null,
    Reserved,
    Empty,
}

impl PrimType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "nat" => Some(PrimType::Nat),
            "int" => Some(PrimType::Int),
            "bool" => Some(PrimType::Bool),
            "text" => Some(PrimType::Text),
            "null" => Some(PrimType::Null),
            "reserved" => Some(PrimType::Reserved),
            "empty" => Some(PrimType::Empty),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum FuncMode {
    Oneway,
    Query,
}

#[derive(Debug)]
pub struct FuncType {
    pub modes: Vec<FuncMode>,
    pub args: Vec<IDLType>,
    pub rets: Vec<IDLType>,
}

#[derive(Debug)]
pub enum Label {
    Id(u32),
    Named(String),
    Unnamed(u32),
}

#[derive(Debug)]
pub struct TypeField {
    pub label: Label,
    pub typ: IDLType,
}

#[derive(Debug)]
pub enum Dec {
    TypD(Binding),
    ImportD(String),
}

#[derive(Debug)]
pub struct Binding {
    pub id: String,
    pub typ: IDLType,
}

#[derive(Debug)]
pub struct IDLProg {
    pub decs: Vec<Dec>,
    pub actor: Option<Binding>,
}

impl IDLProg {
    pub fn to_doc(&self) -> Doc<BoxDoc<()>> {
        let doc = Doc::concat(
            self.decs
                .iter()
                .map(|d| d.to_doc().append(Doc::text(";").append(Doc::newline()))),
        );
        if self.actor.is_some() {
            let actor = self.actor.as_ref().unwrap();
            let doc = doc.append(Doc::text(format!("service {} ", actor.id)));
            match actor.typ {
                IDLType::VarT(ref var) => doc.append(Doc::text(format!(": {}", var))),
                IDLType::ServT(ref meths) => doc.append(meths_to_doc(meths)),
                _ => unreachable!(),
            }
        } else {
            doc
        }
    }
    pub fn to_pretty(&self, width: usize) -> String {
        let mut w = Vec::new();
        self.to_doc().render(width, &mut w).unwrap();
        String::from_utf8(w).unwrap()
    }
}

impl Dec {
    pub fn to_doc(&self) -> Doc<BoxDoc<()>> {
        match *self {
            Dec::TypD(ref b) => Doc::text("type ").append(b.to_doc()),
            Dec::ImportD(ref file) => Doc::text(format!("import \"{}\"", file)),
        }
    }
}

impl Binding {
    pub fn to_doc(&self) -> Doc<BoxDoc<()>> {
        Doc::text(format!("{} =", self.id))
            .append(Doc::space())
            .append(self.typ.to_doc())
            .nest(2)
            .group() // good
    }
}

impl IDLType {
    pub fn to_doc(&self) -> Doc<BoxDoc<()>> {
        match self {
            IDLType::PrimT(p) => Doc::as_string(format!("{:?}", p)),
            IDLType::VarT(var) => Doc::text(var),
            IDLType::FuncT(func) => Doc::text("func").append(Doc::space()).append(func.to_doc()),
            IDLType::OptT(ref t) => Doc::text("opt").append(Doc::space()).append(t.to_doc()),
            IDLType::VecT(ref t) => Doc::text("vec").append(Doc::space()).append(t.to_doc()),
            IDLType::RecordT(ref fs) => Doc::text("record")
                .append(Doc::space())
                .append(fields_to_doc(fs)),
            IDLType::VariantT(ref fs) => Doc::text("variant")
                .append(Doc::space())
                .append(fields_to_doc(fs)),
            IDLType::ServT(ref serv) => Doc::text("service")
                .append(Doc::space())
                .append(meths_to_doc(serv)),
        }
        .nest(2)
        .group()
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Label::Id(n) => write!(f, "{}", n),
            Label::Named(name) => write!(f, "{}", name),
            Label::Unnamed(n) => write!(f, "{}", n),
        }
    }
}

impl TypeField {
    fn to_doc(&self) -> Doc<BoxDoc<()>> {
        Doc::as_string(&self.label)
            .append(Doc::text(":"))
            .append(Doc::space())
            .append(self.typ.to_doc())
            .nest(2)
            .group() // good
    }
}

fn fields_to_doc(fields: &[TypeField]) -> Doc<BoxDoc<()>> {
    Doc::text("{")
        .append(
            Doc::concat(
                fields
                    .iter()
                    .map(|f| Doc::space().append(f.to_doc()).append(Doc::text(";"))),
            )
            .nest(2)
            .group(),
        )
        .append(Doc::text("}"))
}

fn meths_to_doc(meths: &[Binding]) -> Doc<BoxDoc<()>> {
    Doc::text("{")
        .append(
            Doc::concat(meths.iter().map(|meth| {
                let doc =
                    Doc::space().append(Doc::text(format!("{}:", meth.id)).append(Doc::space()));
                let doc = match meth.typ {
                    IDLType::VarT(ref var) => doc.append(Doc::text(var)),
                    IDLType::FuncT(ref func) => doc.append(func.to_doc()),
                    _ => unreachable!(),
                }
                .nest(2)
                .group(); // good
                doc.append(Doc::text(";"))
            }))
            .group(),
        )
        .append(Doc::text("}"))
}

fn args_to_doc(args: &[IDLType]) -> Doc<BoxDoc<()>> {
    Doc::text("(")
        .append(
            Doc::intersperse(
                args.iter().map(|arg| arg.to_doc()),
                Doc::text(",").append(Doc::space()),
            )
            .group(),
        )
        .append(")")
}

impl FuncType {
    fn to_doc(&self) -> Doc<BoxDoc<()>> {
        args_to_doc(&self.args)
            .append(Doc::space())
            .append(Doc::text("->"))
            .append(Doc::space())
            .append(args_to_doc(&self.rets))
            .append(Doc::concat(
                self.modes.iter().map(|m| Doc::text(format!(" {:?}", m))),
            ))
    }
}
