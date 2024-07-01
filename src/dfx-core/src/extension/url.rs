use url::Url;

pub struct ExtensionJsonUrl(Url);

impl ExtensionJsonUrl {
    pub fn registered(name: &str) -> Result<Self, url::ParseError> {
        let s = format!(
            "https://raw.githubusercontent.com/dfinity/dfx-extensions/main/extensions/{name}/extension.json"
        );
        Url::parse(&s).map(ExtensionJsonUrl)
    }

    pub fn to_dependencies_json(&self) -> Result<Url, url::ParseError> {
        self.as_url().join("dependencies.json")
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }
}
