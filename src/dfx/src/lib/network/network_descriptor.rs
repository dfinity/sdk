use crate::config::dfinity::NetworkType;

#[derive(Clone, Debug)]
pub struct NetworkDescriptor {
    pub name: String,
    pub providers: Vec<String>,
    pub r#type: NetworkType,
}
