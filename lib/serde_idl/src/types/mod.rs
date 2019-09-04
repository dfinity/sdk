use reflection::{Reflection, Type as RType};

#[derive(Debug, PartialEq)]
pub enum Type {
    Null,
    Bool,
    Nat,
    Int,
    Text,
    Opt(Box<Type>),
    Vec(Box<Type>),
    Record(Vec<Field>),
    Variant(Vec<Field>),
}

#[derive(Debug, PartialEq)]
pub struct Field {
    id: String,
    ty: Type,
}

pub fn type_of<T>(_v: &T) -> Type where T: Reflection {
    transform(&T::ty(), &T::members())
}

fn transform(ty: &RType, members: &reflection::Schemas) -> Type {
    match ty {
        RType::Bool => Type::Bool,
        RType::I32 => Type::Int,
        RType::U32 => Type::Nat,
        RType::String => Type::Text,
        RType::Struct => {
            let fs = members.iter().map(|f| {
                match f.data {
                    reflection::Member::Field(ref field) => {
                        Field {
                            id: field.id.to_string(),
                            ty: transform(&field.ty, &field.expander.unwrap()())
                        }
                    },
                    _ => unimplemented!("TODO"),
                }
            }).collect();
            Type::Record(fs)
        },/*
        RType::Enum => {
            let fs = members.iter().map(|f| {
                match f.data {
                    reflection::Member::Variant(ref variant) => {
                        Field {
                            id: variant.id.to_string(),
                            ty: transform(&field.ty, &field.expander.unwrap()())
                        }
                    },
                    _ => unimplemented!("TODO"),
                }
            }).collect();
            Type::Record(fs)
        },        */
        _ => unimplemented!("TODO")
    }
}
