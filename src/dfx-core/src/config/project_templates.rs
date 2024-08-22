use itertools::Itertools;
use std::collections::BTreeMap;
use std::io;
use std::sync::OnceLock;

type GetArchiveFn = fn() -> Result<tar::Archive<flate2::read::GzDecoder<&'static [u8]>>, io::Error>;

#[derive(Debug, Clone)]
pub enum ResourceLocation {
    Bundled { get_archive_fn: GetArchiveFn },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Category {
    Backend,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProjectTemplateName(pub String);

#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    pub name: ProjectTemplateName,
    pub display: String,
    pub resource_location: ResourceLocation,
    pub category: Category,
    pub sort_order: u32,
}

type ProjectTemplates = BTreeMap<ProjectTemplateName, ProjectTemplate>;

static PROJECT_TEMPLATES: OnceLock<ProjectTemplates> = OnceLock::new();

pub fn populate(builtin_templates: Vec<ProjectTemplate>) {
    let templates = builtin_templates
        .iter()
        .map(|t| (t.name.clone(), t.clone()))
        .collect();

    PROJECT_TEMPLATES.set(templates).unwrap();
}

pub fn get_project_template(name: &ProjectTemplateName) -> ProjectTemplate {
    PROJECT_TEMPLATES.get().unwrap().get(name).cloned().unwrap()
}

pub fn get_sorted_templates(category: Category) -> Vec<ProjectTemplate> {
    PROJECT_TEMPLATES
        .get()
        .unwrap()
        .values()
        .filter(|t| t.category == category)
        .cloned()
        .sorted_by(|a, b| {
            a.sort_order
                .cmp(&b.sort_order)
                .then_with(|| a.display.cmp(&b.display))
        })
        .collect()
}

pub fn project_template_cli_names(category: Category) -> Vec<String> {
    PROJECT_TEMPLATES
        .get()
        .unwrap()
        .values()
        .filter(|t| t.category == category)
        .map(|t| t.name.0.clone())
        .collect()
}
