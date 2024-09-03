use crate::util::assets;
use dfx_core::config::project_templates::{
    Category, ProjectTemplate, ProjectTemplateName, ResourceLocation,
};

pub fn builtin_templates() -> Vec<ProjectTemplate> {
    let motoko = ProjectTemplate {
        name: ProjectTemplateName("motoko".to_string()),
        display: "Motoko".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_motoko_files,
        },
        category: Category::Backend,
        sort_order: 0,
        update_cargo_lockfile: false,
        has_js: false,
    };

    let rust = ProjectTemplate {
        name: ProjectTemplateName("rust".to_string()),
        display: "Rust".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_rust_files,
        },
        category: Category::Backend,
        sort_order: 1,
        update_cargo_lockfile: true,
        has_js: false,
    };

    let azle = ProjectTemplate {
        name: ProjectTemplateName("azle".to_string()),
        display: "Typescript (Azle)".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_azle_files,
        },
        category: Category::Backend,
        sort_order: 2,
        update_cargo_lockfile: false,
        has_js: true,
    };

    let kybra = ProjectTemplate {
        name: ProjectTemplateName("kybra".to_string()),
        display: "Python (Kybra)".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_kybra_files,
        },
        category: Category::Backend,
        sort_order: 2,
        update_cargo_lockfile: false,
        has_js: false,
    };

    vec![motoko, rust, azle, kybra]
}
