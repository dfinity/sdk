use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProjectTemplateCategory {
    Backend,
    Frontend,
    #[serde(rename = "frontend-test")]
    FrontendTest,
    Extra,
    Support,
}
