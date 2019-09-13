use clap::App;

macro_rules! define_command {
    ($name: ident, $yaml_path: expr) => {
        pub fn $name() -> App<'static, 'static> {
            static mut APP: Option<&'static App<'static, 'static>> = None;

            unsafe {
                match APP {
                    None => {
                        // This is the same code as the load_yaml!() macro except it
                        // does not return a reference but the full ownership of the
                        // yaml object, which we need for the Box::new().
                        // The reason we need to create the app here and cache it
                        // instead of caching only the yaml object is that the clap
                        // crate does not re-export its implementation of the yaml-rust
                        // crate, and we need the proper typing to store it. Their
                        // yaml-rust version is also very old, so we don't want to add
                        // a dependency from us to them.
                        APP = Some(Box::leak(Box::new(App::from_yaml(Box::leak(Box::new(
                            clap::YamlLoader::load_from_str(include_str!($yaml_path))
                                .expect("failed to load YAML file")[0]
                                .clone(),
                        ))))));
                        $name()
                    }
                    Some(app) => app.clone(),
                }
            }
        }
    };
}

define_command!(dfx, "../../assets/command_defs/dfx.yaml");
define_command!(build, "../../assets/command_defs/build.yaml");
define_command!(call, "../../assets/command_defs/call.yaml");
define_command!(start, "../../assets/command_defs/start.yaml");
