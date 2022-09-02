use crate::lib::error::DfxResult;
use crate::lib::project::import::ImportNetworkMapping;

use anyhow::anyhow;

pub fn get_network_mappings(input: &[String]) -> DfxResult<Vec<ImportNetworkMapping>> {
    input
        .iter()
        .map(|v| {
            if let Some(index) = v.find('=') {
                if index == 0 {
                    Err(anyhow!(
                        "malformed network mapping '{}': first network name is empty",
                        &v
                    ))
                } else if index == v.len() - 1 {
                    Err(anyhow!(
                        "malformed network mapping '{}': second network name is empty",
                        &v
                    ))
                } else {
                    Ok(ImportNetworkMapping {
                        network_name_in_this_project: v[..index].to_string(),
                        network_name_in_project_being_imported: v[index + 1..].to_string(),
                    })
                }
            } else {
                Ok(ImportNetworkMapping {
                    network_name_in_this_project: v.clone(),
                    network_name_in_project_being_imported: v.clone(),
                })
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::lib::project::import::ImportNetworkMapping;
    use crate::lib::project::network_mappings::get_network_mappings;

    #[test]
    fn usual() {
        assert_eq!(
            get_network_mappings(&["ic".to_string()]).unwrap(),
            vec![ImportNetworkMapping {
                network_name_in_this_project: "ic".to_string(),
                network_name_in_project_being_imported: "ic".to_string(),
            }],
        );
    }

    #[test]
    fn mapped() {
        assert_eq!(
            get_network_mappings(&["abc=defg".to_string()]).unwrap(),
            vec![ImportNetworkMapping {
                network_name_in_this_project: "abc".to_string(),
                network_name_in_project_being_imported: "defg".to_string(),
            }],
        );
    }

    #[test]
    fn multiple() {
        assert_eq!(
            get_network_mappings(&["abc=defg".to_string(), "ghi=xyz".to_string()]).unwrap(),
            vec![
                ImportNetworkMapping {
                    network_name_in_this_project: "abc".to_string(),
                    network_name_in_project_being_imported: "defg".to_string(),
                },
                ImportNetworkMapping {
                    network_name_in_this_project: "ghi".to_string(),
                    network_name_in_project_being_imported: "xyz".to_string(),
                }
            ],
        );
    }

    #[test]
    #[should_panic(expected = "malformed network mapping '=defg': first network name is empty")]
    fn malformed_missing_first() {
        get_network_mappings(&["=defg".to_string()]).unwrap();
    }

    #[test]
    #[should_panic(expected = "malformed network mapping 'abc=': second network name is empty")]
    fn malformed_missing_second() {
        get_network_mappings(&["abc=".to_string()]).unwrap();
    }
}
