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

impl fmt::Display for IDLProg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for d in self.decs.iter() {
            write!(f, "{};\n", d)?;
        }
        if self.actor.is_some() {
            let actor = self.actor.as_ref().unwrap();
            write!(f, "service {}", actor.id)?;
            match actor.typ {
                IDLType::VarT(ref var) => write!(f, " : {}\n", var)?,
                IDLType::ServT(ref meths) => {
                    write!(f, "{{\n")?;
                    for meth in meths.iter() {
                        write!(f, "{} : {};\n", meth.id, meth.typ)?;
                    }
                    write!(f, "}}\n")?;
                }
                _ => return Err(fmt::Error),
            }
        }
        Ok(())
    }
}

impl fmt::Display for Dec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Dec::TypD(ref b) => write!(f, "type {}", b),
            Dec::ImportD(ref file) => write!(f, "import \"{}\"", file),
        }
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} = {}", self.id, self.typ)
    }
}

impl fmt::Display for IDLType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IDLType::PrimT(p) => write!(f, "{:?}", p)?,
            IDLType::VarT(v) => write!(f, "{}", v)?,
            IDLType::FuncT(func) => write!(f, "func {}", func)?,
            t => write!(f, "{:?}", t)?,
        }
        Ok(())
    }
}

fn show_args(f: &mut fmt::Formatter<'_>, args: &[IDLType]) -> fmt::Result {
    write!(f, "(")?;
    let len = args.len();
    for i in 0..len {
        write!(f, "{}", args[i])?;
        if i < len - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, ")")
}

impl fmt::Display for FuncType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        show_args(f, &self.args)?;
        write!(f, " -> ")?;
        show_args(f, &self.rets)?;
        for m in self.modes.iter() {
            write!(f, " {:?}", m)?;
        }
        Ok(())
    }
}
