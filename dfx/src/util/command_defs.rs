use clap::App;

macro_rules! define_command {
    ($name: ident, $yaml_path: expr) => {
        pub fn $name() -> App<'static, 'static> {
            static mut APP: Option<&'static App<'static, 'static>> = None;

            unsafe {
                match APP {
                    None => {
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
