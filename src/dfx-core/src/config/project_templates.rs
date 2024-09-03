use itertools::Itertools;
use std::collections::BTreeMap;
use std::io;
use std::sync::OnceLock;

type GetArchiveFn = fn() -> Result<tar::Archive<flate2::read::GzDecoder<&'static [u8]>>, io::Error>;

#[derive(Debug, Clone)]
pub enum ResourceLocation {
    /// The template's assets are compiled into the dfx binary
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
    /// The name of the template as specified on the command line,
    /// for example `--type rust`
    pub name: ProjectTemplateName,

    /// The name used for display and sorting
    pub display: String,

    /// How to obtain the template's files
    pub resource_location: ResourceLocation,

    /// Used to determine which CLI group (`--type`, `--backend`, `--frontend`)
    /// as well as for interactive selection
    pub category: Category,

    /// If true, run `cargo update` after creating the project
    pub update_cargo_lockfile: bool,

    /// If true, patch in the any_js template files
    pub has_js: bool,

    /// The sort order is fixed rather than settable in properties:
    /// - motoko=0
    /// - rust=1
    /// - everything else=2 (and then by display name)
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
