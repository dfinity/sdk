use crate::config::dfinity::NetworkType;
use crate::config::dfinity::{DEFAULT_IC_GATEWAY, DEFAULT_IC_GATEWAY_TRAILING_SLASH};

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
        let name_match = matches!(
            network_name,
            "ic" | DEFAULT_IC_GATEWAY | DEFAULT_IC_GATEWAY_TRAILING_SLASH
        );
        let provider_match = {
            providers.len() == 1
                && matches!(
                    providers.get(0).unwrap().as_str(),
                    DEFAULT_IC_GATEWAY | DEFAULT_IC_GATEWAY_TRAILING_SLASH
                )
        };
        name_match || provider_match
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ic_by_netname() {
        assert!(NetworkDescriptor::is_ic("ic", &vec![]));
        assert!(NetworkDescriptor::is_ic(DEFAULT_IC_GATEWAY, &vec![]));
        assert!(NetworkDescriptor::is_ic(
            DEFAULT_IC_GATEWAY_TRAILING_SLASH,
            &vec![]
        ));
    }

    #[test]
    fn ic_by_provider() {
        assert!(NetworkDescriptor::is_ic(
            "not_ic",
            &vec![DEFAULT_IC_GATEWAY.to_string()]
        ));
        assert!(NetworkDescriptor::is_ic(
            "not_ic",
            &vec![DEFAULT_IC_GATEWAY_TRAILING_SLASH.to_string()]
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
        assert!(!NetworkDescriptor::is_ic(
            "not_ic",
            &vec![
                DEFAULT_IC_GATEWAY.to_string(),
                "some_other_provider".to_string()
            ]
        ));
    }
}
