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
        install_node_dependencies: false,
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
        install_node_dependencies: false,
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
        install_node_dependencies: false,
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
        install_node_dependencies: false,
    };

    let sveltekit = ProjectTemplate {
        name: ProjectTemplateName("sveltekit".to_string()),
        display: "SvelteKit".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_svelte_files,
        },
        category: Category::Frontend,
        sort_order: 0,
        update_cargo_lockfile: false,
        has_js: true,
        install_node_dependencies: true,
    };

    let react = ProjectTemplate {
        name: ProjectTemplateName("react".to_string()),
        display: "React".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_react_files,
        },
        category: Category::Frontend,
        sort_order: 1,
        update_cargo_lockfile: false,
        has_js: true,
        install_node_dependencies: true,
    };

    let vue = ProjectTemplate {
        name: ProjectTemplateName("vue".to_string()),
        display: "Vue".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_vue_files,
        },
        category: Category::Frontend,
        sort_order: 2,
        update_cargo_lockfile: false,
        has_js: true,
        install_node_dependencies: true,
    };

    let vanilla = ProjectTemplate {
        name: ProjectTemplateName("vanilla".to_string()),
        display: "Vanilla JS".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_vanillajs_files,
        },
        category: Category::Frontend,
        sort_order: 3,
        update_cargo_lockfile: false,
        has_js: true,
        install_node_dependencies: true,
    };

    let simple_assets = ProjectTemplate {
        name: ProjectTemplateName("simple-assets".to_string()),
        display: "No JS template".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_assets_files,
        },
        category: Category::Frontend,
        sort_order: 4,
        update_cargo_lockfile: false,
        has_js: false,
        install_node_dependencies: false,
    };

    let sveltekit_tests = ProjectTemplate {
        name: ProjectTemplateName("sveltekit-tests".to_string()),
        display: "SvelteKit Test Files".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_svelte_test_files,
        },
        category: Category::FrontendTest,
        sort_order: 0,
        update_cargo_lockfile: false,
        has_js: false,
        install_node_dependencies: false,
    };

    let react_tests = ProjectTemplate {
        name: ProjectTemplateName("react-tests".to_string()),
        display: "React Test Files".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_react_test_files,
        },
        category: Category::FrontendTest,
        sort_order: 0,
        update_cargo_lockfile: false,
        has_js: false,
        install_node_dependencies: false,
    };

    let vue_tests = ProjectTemplate {
        name: ProjectTemplateName("vue-tests".to_string()),
        display: "Vue Test Files".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_vue_test_files,
        },
        category: Category::FrontendTest,
        sort_order: 0,
        update_cargo_lockfile: false,
        has_js: false,
        install_node_dependencies: false,
    };

    let vanillajs_tests = ProjectTemplate {
        name: ProjectTemplateName("vanilla-tests".to_string()),
        display: "Vanilla JS Test Files".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_vanillajs_test_files,
        },
        category: Category::FrontendTest,
        sort_order: 0,
        update_cargo_lockfile: false,
        has_js: false,
        install_node_dependencies: false,
    };

    let internet_identity = ProjectTemplate {
        name: ProjectTemplateName("internet-identity".to_string()),
        display: "Internet Identity".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_internet_identity_files,
        },
        category: Category::Extra,
        sort_order: 0,
        update_cargo_lockfile: false,
        has_js: false,
        install_node_dependencies: false,
    };

    let bitcoin = ProjectTemplate {
        name: ProjectTemplateName("bitcoin".to_string()),
        display: "Bitcoin (Regtest)".to_string(),
        resource_location: ResourceLocation::Bundled {
            get_archive_fn: assets::new_project_bitcoin_files,
        },
        category: Category::Extra,
        sort_order: 1,
        update_cargo_lockfile: false,
        has_js: false,
        install_node_dependencies: false,
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
    ]
}
