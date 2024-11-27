use crate::config::model::project_template::ProjectTemplateCategory;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

type GetArchiveFn = fn() -> Result<tar::Archive<flate2::read::GzDecoder<&'static [u8]>>, io::Error>;

#[derive(Debug, Clone)]
pub enum ResourceLocation {
    /// The template's assets are compiled into the dfx binary
    Bundled { get_archive_fn: GetArchiveFn },

    /// The templates assets are in a directory on the filesystem
    Directory { path: PathBuf },
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProjectTemplateName(pub String);

impl Display for ProjectTemplateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
    pub category: ProjectTemplateCategory,

    /// Other project templates to patch in alongside this one
    pub requirements: Vec<ProjectTemplateName>,

    /// Run a command after adding the canister to dfx.json
    pub post_create: Vec<String>,

    /// If set, display a spinner while this command runs
    pub post_create_spinner_message: Option<String>,

    /// If the post-create command fails, display this warning but don't fail
    pub post_create_failure_warning: Option<String>,

    /// The sort order is fixed rather than settable in properties:
    /// For backend:
    ///   - motoko=0
    ///   - rust=1
    ///   - everything else=2 (and then by display name)
    /// For frontend:
    ///   - SvelteKit=0
    ///   - React=1
    ///   - Vue=2
    ///   - Vanilla JS=3
    ///   - No JS Template=4
    ///   - everything else=5 (and then by display name)
    /// For extras:
    ///   - Internet Identity=0
    ///   - Bitcoin=1
    ///   - everything else=2 (and then by display name)
    ///   - Frontend Tests
    pub sort_order: u32,
}

type ProjectTemplates = BTreeMap<ProjectTemplateName, ProjectTemplate>;

static PROJECT_TEMPLATES: OnceLock<ProjectTemplates> = OnceLock::new();

pub fn populate(builtin_templates: Vec<ProjectTemplate>, loaded_templates: Vec<ProjectTemplate>) {
    let templates: ProjectTemplates = builtin_templates
        .into_iter()
        .map(|t| (t.name.clone(), t))
        .chain(loaded_templates.into_iter().map(|t| (t.name.clone(), t)))
        .collect();

    PROJECT_TEMPLATES.set(templates).unwrap();
}

pub fn get_project_template(name: &ProjectTemplateName) -> ProjectTemplate {
    PROJECT_TEMPLATES.get().unwrap().get(name).cloned().unwrap()
}

pub fn find_project_template(name: &ProjectTemplateName) -> Option<ProjectTemplate> {
    PROJECT_TEMPLATES.get().unwrap().get(name).cloned()
}

pub fn get_sorted_templates(category: ProjectTemplateCategory) -> Vec<ProjectTemplate> {
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

pub fn project_template_cli_names(category: ProjectTemplateCategory) -> Vec<String> {
    PROJECT_TEMPLATES
        .get()
        .unwrap()
        .values()
        .filter(|t| t.category == category)
        .map(|t| t.name.0.clone())
        .collect()
}
