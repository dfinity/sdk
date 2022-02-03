use crate::config::dfinity::NetworkType;
use crate::config::dfinity::DEFAULT_IC_GATEWAY;

#[derive(Clone, Debug)]
pub struct NetworkDescriptor {
    pub name: String,
    pub providers: Vec<String>,
    pub r#type: NetworkType,
    pub is_ic: bool,
}

impl NetworkDescriptor {
    // Determines whether the provided connection is the official IC or not.
    #[allow(clippy::ptr_arg)]
    pub fn is_ic(network_name: &str, providers: &Vec<String>) -> bool {
        let name_match = network_name == "ic" || network_name == DEFAULT_IC_GATEWAY;
        let provider_match =
            { providers.len() == 1 && providers.get(0).unwrap() == "https://ic0.app" };
        name_match || provider_match
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ic_by_netname() {
        assert!(NetworkDescriptor::is_ic("ic", &vec![]));
    }

    #[test]
    fn ic_by_provider() {
        assert!(!NetworkDescriptor::is_ic(
            "not_ic",
            &vec!["https://ic0.app".to_string()]
        ));
    }

    #[test]
    fn ic_by_netname_fail() {
        assert!(!NetworkDescriptor::is_ic("not_ic", &vec![]));
    }

    #[test]
    fn ic_by_provider_fail_string() {
        assert!(!NetworkDescriptor::is_ic(
            "not_ic",
            &vec!["not_ic_provider".to_string()]
        ));
    }

    #[test]
    fn ic_by_provider_fail_unique() {
        assert!(NetworkDescriptor::is_ic(
            "not_ic",
            &vec![
                "https://ic0.app".to_string(),
                "some_other_provider".to_string()
            ]
        ));
    }
}
