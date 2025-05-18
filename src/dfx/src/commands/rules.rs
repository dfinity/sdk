use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::iter::once;
use std::path::Path;
use std::vec;

use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::builders::CanisterBuilder;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::graph::graph_nodes_map::GraphWithNodesMap;
use crate::lib::models::canister::{CanisterPool, Import};
use crate::lib::builders::custom::CustomBuilder;
use crate::lib::network::network_opt::NetworkOpt;
use itertools::Itertools;
use dfx_core::config::model::dfinity::{CanisterTypeProperties, ConfigCanistersCanister};
use clap::Parser;
use petgraph::visit::EdgeRef;
use petgraph::Graph;
use petgraph::visit::GraphBase;

/// Output dependencies in Make format
#[derive(Parser)]
pub struct RulesOpts {
    /// File to output make rules
    #[arg(long, short, value_name = "FILE")]
    output: Option<String>,

    #[clap(flatten)]
    network: NetworkOpt,
}

mod elements {
    use std::fmt::{Display, Formatter};
    use itertools::Itertools;

    pub trait Target: Display {
        fn is_phony(&self) -> bool;
    }

    impl<T: Target> Target for Box<T> {
        fn is_phony(&self) -> bool {
            self.as_ref().is_phony()
        }
    }

    pub struct File(pub String);

    impl Display for File {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }        
    }

    impl Target for File {
        fn is_phony(&self) -> bool {
            false
        }
    }

    pub struct PhonyTarget(pub String);

    impl Display for PhonyTarget {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Target for PhonyTarget {
        fn is_phony(&self) -> bool {
            true
        }
    }

    /// "Elements" of rules file separated by empty lines.
    pub trait Element: Display {}

    pub struct Rule {
        pub targets: Vec<Box<dyn Target>>, // If targets contain files, use `DoubleRule` instead.
        pub sources: Vec<Box<dyn Target>>,
        pub commands: Vec<String>,
    }

    impl Element for Rule {}

    impl Display for Rule {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let targets_str = self.targets.iter().map(|t| t.to_string()).join(" ");
            let sources_str = self.sources.iter().map(|t| t.to_string()).join(" ");
            let phony_targets: Vec<&Box<dyn Target>> = self.targets
                .iter().filter(|target| target.is_phony())
                .collect();
            if !phony_targets.is_empty() {
                write!(f, ".PHONY: {}\n", phony_targets.iter().join(" "))?;
            }
            write!(f, "{}: {}", targets_str, sources_str)?;
            for command in &self.commands {
                write!(f, "\n\t{}", command)?;
            }
            Ok(())
        }
    }

    /// ```
    /// phony: target1 target2
    /// target1 target2: source1 source2
    /// ```
    pub struct DoubleRule {
        pub phony: PhonyTarget,
        pub targets: Vec<File>,
        pub sources: Vec<Box<dyn Target>>,
        pub commands: Vec<String>,
    }

    impl Element for DoubleRule {}

    impl Display for DoubleRule {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let targets_str = self.targets.iter().map(|t| t.to_string()).join(" ");
            let sources_str = self.sources.iter().map(|t| t.to_string()).join(" ");
            write!(f, ".PHONY: {}\n", self.phony)?;
            write!(f, "{}: ", self.phony)?;
            write!(f, "{}\n\n", targets_str)?;
            write!(f, "{}: ", targets_str)?;
            write!(f, "{}", sources_str)?;
            for command in &self.commands {
                write!(f, "\n\t{}", command)?;
            }
                Ok(())
        }
    }
}

// FIXME: "remote" (and not only) canisters build two times.
// FIXME: It wrongly acts with downloaded canisters (like `internet_identity`).
//        This seems to be the cause of double recompilation. (Seems to have been fixed.)
// TODO: Describe explicity rules and files. Correct outputting \n between rules.
pub fn exec(env1: &dyn Environment, opts: RulesOpts) -> DfxResult {
    let env = create_anonymous_agent_environment(env1, opts.network.to_network_name())?;
    // let log = env.get_logger();

    // Read the config.
    let config = env.get_config_or_anyhow()?;

    // We load dependencies before creating the file to minimize the time that the file is half-written.
    // Load dependencies for Make rules:
    let builder = CustomBuilder::new(&env)?; // hackish use of CustomBuilder not intended for this use
    let canisters = &config.get_config().canisters.as_ref();
    let canister_names = if let Some(canisters) = canisters {
        canisters.keys().map(|k| k.to_string()).collect::<Vec<String>>()
    } else {
        Vec::new()
    };
    let pool: CanisterPool = CanisterPool::load(
        &env, // if `env1`,  fails with "NetworkDescriptor only available from an AgentEnvironment"
        false,
        &canister_names,
    )?;
    builder.read_all_dependencies(
        &env,
        &pool,
    )?;

    let mut rules = Vec::<Box<dyn elements::Element>>::new();

    let graph0 = env.get_imports().borrow();
    let graph = graph0.graph();

    match &canisters {
        Some(canisters) => {
            let canisters: &BTreeMap<String, ConfigCanistersCanister> = canisters;
            for canister in canisters {
                // duplicate code
                let canister2: std::sync::Arc<crate::lib::models::canister::Canister> = pool.get_first_canister_with_name(&canister.0).unwrap();
                let (targets, source) = if canister2.get_info().is_assets() {
                    let path1 = format!(".dfx/$(NETWORK)/canisters/{}/assetstorage.wasm.gz", canister.0);
                    (vec![elements::File(path1)], Vec::new())
                /*} else if canister2.get_info().is_remote() {
                    (vec![elements::File(format!("candid/{}.did", canister.0))], Vec::new()) // FIXME: It is created by `dfx nns import`.*/
                } else {
                    // TODO: `graph` here is superfluous:
                    let path = make_targets(&pool, &graph0, graph, *graph0.nodes().get(&Import::Canister(canister.0.clone())).unwrap())?; // TODO: `unwrap`?
                    (
                        path.into_iter().collect(),
                        if let Some(main) = &canister.1.main {
                            vec![elements::File(main.to_str().unwrap().to_string())]
                        } else {
                            Vec::new()
                        }
                    )
                };
                let build_command = format!("dfx build --no-deps --network $(NETWORK) {}", canister.0);
                rules.push(Box::new(elements::DoubleRule { // FIXME
                    phony: elements::PhonyTarget(format!("build@{}", canister.0)),
                    targets,
                    sources: source.into_iter().map(|t| Box::new(t) as Box<dyn elements::Target>).collect(),
                    commands: 
                        if canister2.get_info().is_remote() { // FIXME: correct?
                            vec![
                                build_command,
                            ]
                        } else {
                            vec![
                                format!("dfx canister create --network $(NETWORK) {}", canister.0),
                                build_command,
                            ]
                        }
                }));
            };
            for canister in canisters {
                let declarations_config_pre = &canister.1.declarations;
                // let workspace_root = config.get_path().parent().unwrap();
                // duplicate code:
                let output = declarations_config_pre
                    .output
                    .clone()
                    .unwrap_or_else(|| Path::new("src/declarations").join(canister.0));
                let bindings = declarations_config_pre
                    .bindings
                    .clone() // probably, inefficient
                    .unwrap_or_else(|| vec!["js".to_string(), "ts".to_string(), "did".to_string()]);
                if !bindings.is_empty() {
                    let deps = bindings.iter().map(|lang| {
                        match lang.as_str() {
                            "did" => vec![format!("{}.did", canister.0)],
                            "mo" => vec![format!("{}.mo", canister.0)],
                            "rs" => vec![], // TODO
                            "js" => vec![format!("{}.did.js", canister.0), "index.js".to_string()],
                            "ts" => vec![format!("{}.did.d.ts", canister.0), "index.d.ts".to_string()],
                            _ => panic!("unknown canister type: {}", canister.0.as_str()),
                        }
                    }).flatten().map(|path| elements::File(output.join(path).to_str().unwrap().to_string())); // TODO: `unwrap`
                    let deps = deps.map(|t| Box::new(t) as Box<dyn elements::Target>);
                    if let CanisterTypeProperties::Custom { .. } = &canister.1.type_specific {
                        // TODO
                    } else {
                        let did: String = if let CanisterTypeProperties::Assets { .. } = &canister.1.type_specific {
                            "service.did".to_string()
                        } else {
                            format!("{}.did", canister.0)
                        };
                        rules.push(Box::new(elements::DoubleRule {
                            phony: elements::PhonyTarget(format!("generate@{}", canister.0)),
                            targets: vec![
                                elements::File(format!(".dfx/$(NETWORK)/canisters/{}/{}", canister.0, did)),
                            ],
                            sources: once(
                                Box::new(elements::PhonyTarget(format!("build@{}", canister.0))) as Box<dyn elements::Target>
                            ).chain(deps).collect(),
                            commands: vec![
                                format!("dfx generate --no-compile --network $(NETWORK) {}", canister.0),
                            ],
                        }));
                    }
                }
            };
        }
        None => {}
    };

    for edge in graph.edge_references() {
        let target_value = graph.node_weight(edge.target()).unwrap();
        if let Import::Lib(_) = target_value {
            // Unused, because package manager never update existing files (but create new dirs)
        } else {
            rules.push(Box::new(elements::Rule {
                // Yes, source and target are reversed:
                targets: make_targets(&pool, &graph0, graph, edge.source())?
                    .into_iter().map(|t| Box::new(t) as Box<dyn elements::Target>).collect(),
                sources: make_targets(&pool, &graph0, graph, edge.target())?
                    .into_iter().map(|t| Box::new(t) as Box<dyn elements::Target>).collect(),
                commands: Vec::new(),
            }));
        }
    }
    for node in graph0.nodes() {
        let commands = get_build_commands(&pool, graph, *node.1);
        if let Import::Canister(canister_name) = node.0 {
            let canister: std::sync::Arc<crate::lib::models::canister::Canister> = pool.get_first_canister_with_name(&canister_name).unwrap();
            if !commands.is_empty() {
                let targets = make_targets(&pool, &graph0, graph, *node.1)?;
                if canister.as_ref().get_info().is_assets() {
                    // We don't support generating dependencies for assets,
                    // so recompile it every time:
                    rules.push(Box::new(elements::Rule {
                        targets: targets.into_iter().map(|t| Box::new(t) as Box<dyn elements::Target>).collect(),
                        sources: Vec::new(),
                        commands: commands,
                    }));
                }
            }
            let deps = canister.as_ref().get_info().get_dependencies();
            let commands = if !canister.as_ref().get_info().is_remote() {
                vec![
                    format!( // TODO: Use `canister install` instead.
                        "dfx deploy --no-compile --network $(NETWORK) $(DEPLOY_FLAGS) $(DEPLOY_FLAGS.{}) {}\n\n",
                        canister_name, canister_name
                    ),
                ]   
            } else {
                Vec::new()
            };
            rules.push(Box::new(elements::Rule {
                targets: vec![Box::new(elements::PhonyTarget(format!("deploy-self@{}", canister_name)))],
                sources: vec![Box::new(elements::PhonyTarget(format!("build@{}", canister_name)))],
                commands,
            }));
            // If the canister is assets, add `generate@` dependencies.
            if canister.as_ref().get_info().is_assets() {
                if !deps.is_empty() {
                    rules.push(Box::new(elements::Rule {
                        targets: vec![Box::new(elements::File(format!("build@{}", canister_name))) as Box<dyn elements::Target>],
                        sources: deps.iter().map(|name| Box::new(elements::PhonyTarget(format!("generate@{}", name))) as Box<dyn elements::Target>)
                            .collect(),
                        commands: Vec::new(), // TODO
                    }));
                }
            }
            rules.push(Box::new(elements::Rule {
                targets: vec![Box::new(elements::PhonyTarget(format!("deploy@{}", canister_name)))],
                sources: deps.iter().map(|name| elements::PhonyTarget(format!("deploy@{}", name)))
                    .chain(once(elements::PhonyTarget(format!("deploy-self@{}", canister_name))))
                    .map(|t| Box::new(t) as Box<dyn elements::Target>).collect(),
                commands: Vec::new(),
            }));
        }
    }

    let mut output_file: Box<dyn Write> = match opts.output {
        Some(filename) => Box::new(OpenOptions::new().write(true).create(true).truncate(true).open(filename)?),
        None => Box::new(std::io::stdout()),
    };

    output_file.write_fmt(format_args!("NETWORK ?= local\n\n"))?;
    output_file.write_fmt(format_args!("DEPLOY_FLAGS ?= \n\n"))?;
    output_file.write_fmt(format_args!("ROOT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))\n\n"))?;
    output_file.write_fmt(format_args!("{}", rules.iter().join("\n\n")))?;

    Ok(())
}

/// Return Make targets (files) for the given node.
fn make_targets(
    pool: &CanisterPool,
    graph0: &GraphWithNodesMap<Import, ()>,
    graph: &Graph<Import, ()>,
    node_id: <Graph<Import, ()> as GraphBase>::NodeId
) -> DfxResult<Vec<elements::File>> {
    let node_value = graph.node_weight(node_id).unwrap();
    Ok(match node_value {
        Import::Canister(canister_name) => {
            // duplicate code
            let canister: std::sync::Arc<crate::lib::models::canister::Canister> = pool.get_first_canister_with_name(&canister_name).unwrap();
            if canister.get_info().is_assets() {
                let path1 = format!(".dfx/$(NETWORK)/canisters/{}/assetstorage.wasm.gz", canister_name);
                // let path2 = format!(".dfx/$(NETWORK)/canisters/{}/assetstorage.did", canister_name);
                vec![elements::File(path1)]
            } else if canister.get_info().is_remote() || canister.get_info().is_custom() {
                // let is_gzip = canister.get_info().get_gzip(); // produces `false`, even if `"wasm"` is compressed.
                let is_gzip = // hack
                    if let CanisterTypeProperties::Custom { wasm, .. } = &canister.get_info().get_type_specific_properties() {
                        wasm.ends_with(".gz")
                    } else {
                        canister.get_info().get_gzip()
                    };
                let path1 = if is_gzip {
                    format!(".dfx/$(NETWORK)/canisters/{}/{}.wasm.gz", canister_name, canister_name)
                } else {
                    format!(".dfx/$(NETWORK)/canisters/{}/{}.wasm", canister_name, canister_name)
                };
                let path2 = format!(".dfx/$(NETWORK)/canisters/{}/{}.did", canister_name, canister_name);
                vec![elements::File(path1), elements::File(path2)]
            } else {
                let did = if canister.get_info().is_assets() {
                    "service.did".to_string()
                } else {
                    format!("{}.did", canister_name)
                };
                let path1 = format!(".dfx/$(NETWORK)/canisters/{}/{}.wasm", canister_name, canister_name);
                let path2 = format!(".dfx/$(NETWORK)/canisters/{}/{}", canister_name, did);
                vec![elements::File(path1), elements::File(path2)]
            }
        }
        Import::Path(path) => vec![elements::File(format!("{}", path.to_str().unwrap_or("<unknown>").to_owned()))], // TODO: <unknown> is a hack
        Import::Ic(canister_name) => {
            // format!("build@{}", canister_name)
            let canister2: std::sync::Arc<crate::lib::models::canister::Canister> = pool.get_first_canister_with_name(&canister_name).unwrap();
            if canister2.get_info().is_assets() {
                let path1 = vec![elements::File(format!(".dfx/$(NETWORK)/canisters/{}/assetstorage.wasm.gz", canister_name))];
                path1
            } else {
                // TODO: `graph` here is superfluous:
                let path = make_targets(&pool, &graph0, graph, *graph0.nodes().get(&Import::Canister(canister_name.clone())).unwrap())?; // TODO: `unwrap`?
                path
            }
        }
        Import::Lib(_path) => vec![], // TODO: Does it work correctly?
    })
}

fn get_build_commands(pool: &CanisterPool, graph: &Graph<Import, ()>, node_id: <Graph<Import, ()> as GraphBase>::NodeId) -> Vec<String> {
    let node_value = graph.node_weight(node_id).unwrap();
    match node_value {
        Import::Canister(canister_name) => {
            // TODO: Duplicate code in next line:
            let canister: std::sync::Arc<crate::lib::models::canister::Canister> = pool.get_first_canister_with_name(&canister_name).unwrap();
            let last_line = format!("dfx build --no-deps --network $(NETWORK) {}", canister_name);
            if canister.get_info().is_remote() {
                vec![last_line]
            } else {
                vec![
                    format!("dfx canister create --network $(NETWORK) {}", canister_name),
                    last_line,
                ]
            }
        }
        Import::Ic(_canister_name) => Vec::new(),
        Import::Path(_path) => Vec::new(),
        Import::Lib(_path) => Vec::new(),
    }
}