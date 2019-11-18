extern crate pretty;
use self::pretty::{BoxDoc, Doc};
use dfx_info::idl_hash;

#[derive(Debug, Clone)]
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

macro_rules! enum_to_doc {
    (pub enum $name:ident {
        $($variant:ident),*,
    }) => {
        #[derive(Debug, Clone)]
        pub enum $name {
            $($variant),*
        }
        impl $name {
            fn to_doc(&self) -> Doc<'_, BoxDoc<'_, ()>> {
                match self {
                    $($name::$variant => Doc::text(stringify!($variant).to_lowercase())),*
                }
            }
            pub fn str_to_enum(str: &str) -> Option<Self> {
                $(if str == stringify!($variant).to_lowercase() {
                    return Some($name::$variant);
                });*
                return None;
            }
        }
    };
}

enum_to_doc! {
pub enum PrimType {
    Nat,
    Nat8,
    Nat16,
    Nat32,
    Nat64,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    Text,
    Null,
    Reserved,
    Empty,
}}

enum_to_doc! {
pub enum FuncMode {
    Oneway,
    Query,
}}

#[derive(Debug, Clone)]
pub struct FuncType {
    pub modes: Vec<FuncMode>,
    pub args: Vec<IDLType>,
    pub rets: Vec<IDLType>,
}

#[derive(Debug, Clone)]
pub enum Label {
    Id(u32),
    Named(String),
    Unnamed(u32),
}

impl Label {
    pub fn get_id(&self) -> u32 {
        match *self {
            Label::Id(n) => n,
            Label::Named(ref n) => idl_hash(n),
            Label::Unnamed(n) => n,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeField {
    pub label: Label,
    pub typ: IDLType,
}

#[derive(Debug)]
pub enum Dec {
    TypD(Binding),
    ImportD(String),
}

#[derive(Debug, Clone)]
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
    pub fn to_doc(&self) -> Doc<'_, BoxDoc<'_, ()>> {
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
    pub fn to_doc(&self) -> Doc<'_, BoxDoc<'_, ()>> {
        match *self {
            Dec::TypD(ref b) => Doc::text("type ").append(b.to_doc()),
            Dec::ImportD(ref file) => Doc::text(format!("import \"{}\"", file)),
        }
    }
}

impl Binding {
    fn to_doc(&self) -> Doc<'_, BoxDoc<'_, ()>> {
        Doc::text(format!("{} =", self.id))
            .append(Doc::space())
            .append(self.typ.to_doc())
            .nest(2)
            .group()
    }
}

impl IDLType {
    pub fn to_doc(&self) -> Doc<'_, BoxDoc<'_, ()>> {
        match self {
            IDLType::PrimT(p) => p.to_doc(),
            IDLType::VarT(var) => Doc::text(var),
            IDLType::FuncT(func) => Doc::text("func").append(Doc::space()).append(func.to_doc()),
            IDLType::OptT(ref t) => Doc::text("opt").append(Doc::space()).append(t.to_doc()),
            IDLType::VecT(ref t) => Doc::text("vec").append(Doc::space()).append(t.to_doc()),
            IDLType::RecordT(ref fs) => Doc::text("record ").append(fields_to_doc(fs)),
            IDLType::VariantT(ref fs) => Doc::text("variant ").append(fields_to_doc(fs)),
            IDLType::ServT(ref serv) => Doc::text("service ").append(meths_to_doc(serv)),
        }
        .nest(2)
        .group()
    }
}

impl FuncType {
    fn to_doc(&self) -> Doc<'_, BoxDoc<'_, ()>> {
        args_to_doc(&self.args)
            .append(Doc::space())
            .append(Doc::text("-> "))
            .append(args_to_doc(&self.rets))
            .append(Doc::concat(
                self.modes.iter().map(|m| Doc::space().append(m.to_doc())),
            ))
    }
}

impl TypeField {
    fn to_doc(&self) -> Doc<'_, BoxDoc<'_, ()>> {
        let colon = Doc::text(":").append(Doc::space());
        let doc = match &self.label {
            Label::Id(n) => Doc::as_string(n).append(colon),
            Label::Named(name) => Doc::text(name).append(colon),
            Label::Unnamed(_) => Doc::nil(),
        };
        doc.append(self.typ.to_doc()).nest(2).group()
    }
}

fn fields_to_doc(fields: &[TypeField]) -> Doc<'_, BoxDoc<'_, ()>> {
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
        .append(Doc::space())
        .append(Doc::text("}"))
}

fn meths_to_doc(meths: &[Binding]) -> Doc<'_, BoxDoc<'_, ()>> {
    Doc::text("{")
        .append(Doc::concat(meths.iter().map(|meth| {
            let doc = Doc::newline().append(Doc::text(format!("{}:", meth.id)));
            let doc = match meth.typ {
                IDLType::VarT(ref var) => doc.append(Doc::space().append(Doc::text(var))),
                IDLType::FuncT(ref func) => doc.append(Doc::space().append(func.to_doc()).nest(2)),
                _ => unreachable!(),
            }
            .nest(2)
            .group();
            doc.append(Doc::text(";"))
        })))
        .append(Doc::space())
        .append(Doc::text("}"))
}

fn args_to_doc(args: &[IDLType]) -> Doc<'_, BoxDoc<'_, ()>> {
    Doc::text("(")
        .append(
            Doc::intersperse(
                args.iter().map(|arg| arg.to_doc()),
                Doc::text(",").append(Doc::space()),
            )
            .nest(1)
            .group(),
        )
        .append(")")
}
