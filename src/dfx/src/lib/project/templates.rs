use crate::util::assets;
use dfx_core::config::project_templates::{
    Category, ProjectTemplate, ProjectTemplateName, ResourceLocation,
};

const NPM_INSTALL: &str = "npm install --quiet --no-progress --workspaces --if-present";
const NPM_INSTALL_SPINNER_MESSAGE: &str = "Installing node dependencies...";
const NPM_INSTALL_FAILURE_WARNING: &str =
    "An error occurred. See the messages above for more details.";
const CARGO_UPDATE_FAILURE_MESSAGE: &str = "You will need to run it yourself (or a similar command like `cargo vendor`), because `dfx build` will use the --locked flag with Cargo.";

pub fn builtin_templates() -> Vec<ProjectTemplate> {
    let motoko = ProjectTemplate {
        name: ProjectTemplateName("motoko".to_string()),
        display: "Motoko".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_motoko_files,
        },
        category: Category::Backend,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 0,
    };

    let rust = ProjectTemplate {
        name: ProjectTemplateName("rust".to_string()),
        display: "Rust".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_rust_files,
        },
        category: Category::Backend,
        post_create: vec!["cargo update".to_string()],
        post_create_failure_warning: Some(CARGO_UPDATE_FAILURE_MESSAGE.to_string()),
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 1,
    };

    let azle = ProjectTemplate {
        name: ProjectTemplateName("azle".to_string()),
        display: "Typescript (Azle)".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_azle_files,
        },
        category: Category::Backend,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![ProjectTemplateName("dfx_js_base".to_string())],
        sort_order: 2,
    };

    let kybra = ProjectTemplate {
        name: ProjectTemplateName("kybra".to_string()),
        display: "Python (Kybra)".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_kybra_files,
        },
        category: Category::Backend,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 2,
    };

    let sveltekit = ProjectTemplate {
        name: ProjectTemplateName("sveltekit".to_string()),
        display: "SvelteKit".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_svelte_files,
        },
        category: Category::Frontend,
        post_create: vec![NPM_INSTALL.to_string()],
        post_create_failure_warning: Some(NPM_INSTALL_FAILURE_WARNING.to_string()),
        post_create_spinner_message: Some(NPM_INSTALL_SPINNER_MESSAGE.to_string()),
        requirements: vec![ProjectTemplateName("dfx_js_base".to_string())],
        sort_order: 0,
    };

    let react = ProjectTemplate {
        name: ProjectTemplateName("react".to_string()),
        display: "React".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_react_files,
        },
        category: Category::Frontend,
        post_create: vec![NPM_INSTALL.to_string()],
        post_create_failure_warning: Some(NPM_INSTALL_FAILURE_WARNING.to_string()),
        post_create_spinner_message: Some(NPM_INSTALL_SPINNER_MESSAGE.to_string()),
        requirements: vec![ProjectTemplateName("dfx_js_base".to_string())],
        sort_order: 1,
    };

    let vue = ProjectTemplate {
        name: ProjectTemplateName("vue".to_string()),
        display: "Vue".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_vue_files,
        },
        category: Category::Frontend,
        post_create: vec![NPM_INSTALL.to_string()],
        post_create_failure_warning: Some(NPM_INSTALL_FAILURE_WARNING.to_string()),
        post_create_spinner_message: Some(NPM_INSTALL_SPINNER_MESSAGE.to_string()),
        requirements: vec![ProjectTemplateName("dfx_js_base".to_string())],
        sort_order: 2,
    };

    let vanilla = ProjectTemplate {
        name: ProjectTemplateName("vanilla".to_string()),
        display: "Vanilla JS".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_vanillajs_files,
        },
        category: Category::Frontend,
        post_create: vec![NPM_INSTALL.to_string()],
        post_create_failure_warning: Some(NPM_INSTALL_FAILURE_WARNING.to_string()),
        post_create_spinner_message: Some(NPM_INSTALL_SPINNER_MESSAGE.to_string()),
        requirements: vec![ProjectTemplateName("dfx_js_base".to_string())],
        sort_order: 3,
    };

    let simple_assets = ProjectTemplate {
        name: ProjectTemplateName("simple-assets".to_string()),
        display: "No JS template".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_assets_files,
        },
        category: Category::Frontend,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 4,
    };

    let sveltekit_tests = ProjectTemplate {
        name: ProjectTemplateName("sveltekit-tests".to_string()),
        display: "SvelteKit Test Files".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_svelte_test_files,
        },
        category: Category::FrontendTest,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 0,
    };

    let react_tests = ProjectTemplate {
        name: ProjectTemplateName("react-tests".to_string()),
        display: "React Test Files".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_react_test_files,
        },
        category: Category::FrontendTest,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 0,
    };

    let vue_tests = ProjectTemplate {
        name: ProjectTemplateName("vue-tests".to_string()),
        display: "Vue Test Files".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_vue_test_files,
        },
        category: Category::FrontendTest,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 0,
    };

    let vanillajs_tests = ProjectTemplate {
        name: ProjectTemplateName("vanilla-tests".to_string()),
        display: "Vanilla JS Test Files".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_vanillajs_test_files,
        },
        category: Category::FrontendTest,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 0,
    };

    let internet_identity = ProjectTemplate {
        name: ProjectTemplateName("internet-identity".to_string()),
        display: "Internet Identity".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_internet_identity_files,
        },
        category: Category::Extra,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 0,
    };

    let bitcoin = ProjectTemplate {
        name: ProjectTemplateName("bitcoin".to_string()),
        display: "Bitcoin (Regtest)".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_bitcoin_files,
        },
        category: Category::Extra,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 1,
    };

    let js_base = ProjectTemplate {
        name: ProjectTemplateName("dfx_js_base".to_string()),
        display: "<never shown>>".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_js_files,
        },
        category: Category::Support,
        post_create: vec![],
        post_create_failure_warning: None,
        post_create_spinner_message: None,
        requirements: vec![],
        sort_order: 2,
    };

    vec![
        motoko,
        rust,
        azle,
        kybra,
        vanilla,
        sveltekit,
        vue,
        react,
        simple_assets,
        sveltekit_tests,
        react_tests,
        vue_tests,
        vanillajs_tests,
        internet_identity,
        bitcoin,
        js_base,
    ]
}
