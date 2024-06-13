#[derive(Clone, Debug)]
pub enum ContentEncoder {
    Gzip,
    Brotli,
}

impl std::fmt::Display for ContentEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ContentEncoder::Gzip => f.write_str("gzip"),
            ContentEncoder::Brotli => f.write_str("br"),
        }
    }
}
