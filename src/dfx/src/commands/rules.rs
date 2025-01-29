use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::builders::CanisterBuilder;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::{CanisterPool, Import};
use crate::lib::builders::custom::CustomBuilder;
use itertools::Itertools;
use dfx_core::config::model::dfinity::ConfigCanistersCanister;
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
}

// TODO: When deploying a canister, deploy its dependendencies, even if `--no-compile``.
pub fn exec(env1: &dyn Environment, opts: RulesOpts) -> DfxResult {
    let env = create_anonymous_agent_environment(env1, None)?;
    // let log = env.get_logger();

    // Read the config.
    let config = env.get_config_or_anyhow()?;

    let env = create_anonymous_agent_environment(env1, None)?;

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

    let mut output_file: Box<dyn Write> = match opts.output {
        Some(filename) => Box::new(OpenOptions::new().write(true).create(true).truncate(true).open(filename)?),
        None => Box::new(std::io::stdout()),
    };

    output_file.write_fmt(format_args!("NETWORK ?= local\n\n"))?;
    output_file.write_fmt(format_args!("DEPLOY_FLAGS ?= \n\n"))?;
    output_file.write_fmt(format_args!("ROOT_DIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))\n\n"))?;

    match &canisters {
        Some(canisters) => {
            let canisters: &BTreeMap<String, ConfigCanistersCanister> = canisters;
            output_file.write_fmt(format_args!(".PHONY:"))?;
            for canister in canisters {
                output_file.write_fmt(format_args!(" canister@{}", canister.0))?;
            };
            output_file.write_fmt(format_args!("\n\n.PHONY:"))?;
            for canister in canisters {
                output_file.write_fmt(format_args!(" deploy@{}", canister.0))?;
            }
            output_file.write_fmt(format_args!("\n\n.PHONY:"))?;
            for canister in canisters {
                output_file.write_fmt(format_args!(" generate@{}", canister.0))?;
            }
            output_file.write_fmt(format_args!("\n\n"))?;
            for canister in canisters {
                // duplicate code
                let canister2 = pool.get_first_canister_with_name(&canister.0).unwrap(); // TODO: `unwrap`
                if canister2.get_info().is_assets() {
                    let path1 = format!("$(ROOT_DIR)/.dfx/local/canisters/{}/assetstorage.wasm.gz", canister.0);
                    let path2 = format!("$(ROOT_DIR)/.dfx/local/canisters/{}/assetstorage.did", canister.0);
                    output_file.write_fmt(format_args!("canister@{}: \\\n  {} {}\n\n", canister.0, path1, path2))?;
                    output_file.write_fmt(format_args!(
                        "{} {}:\n\tdfx canister create {}\n\tdfx build --no-deps {}\n\n", path1, path2, canister.0, canister.0
                    ))?;
                } else {
                    let path1 = format!("$(ROOT_DIR)/.dfx/local/canisters/{}/{}.wasm", canister.0, canister.0);
                    let path2 = format!("$(ROOT_DIR)/.dfx/local/canisters/{}/{}.did", canister.0, canister.0);
                        output_file.write_fmt(format_args!("canister@{}: \\\n  {} {}\n\n", canister.0, path1, path2))?;
                    if let Some(main) = &canister.1.main {
                        output_file.write_fmt(format_args!("{} {}: $(ROOT_DIR)/{}\n\n", path1, path2, main.to_str().unwrap()))?;
                    }
                }
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
                            _ => panic!("unknown canister type: {}", canister.0.as_str()), // TODO
                        }
                    }).flatten().map(|path| format!("$(ROOT_DIR)/{}", output.join(path).to_str().unwrap().to_string())).join(" "); // TODO: `unwrap`
                    output_file.write_fmt(format_args!(
                        "generate@{}: \\\n  {}\n\n",
                        canister.0,
                        deps,
                    ))?;
                    output_file.write_fmt(format_args!(
                        "{}: {}\n\t{} {}\n\n",
                        deps,
                        format!("$(ROOT_DIR)/.dfx/local/canisters/{}/{}.did", canister.0, canister.0),
                        "dfx generate --no-compile --network $(NETWORK)",
                        canister.0,
                    ))?;
                }
            };
        }
        None => {}
    };

    let graph0 = env.get_imports().borrow();
    let graph = graph0.graph();
    for edge in graph.edge_references() {
        let target_value = graph.node_weight(edge.target()).unwrap();
        if let Import::Lib(_) = target_value {
             // TODO: Unused, because package manager never update existing files (but create new dirs)
        } else {
            output_file.write_fmt(format_args!(
                "{}: {}\n",
                make_target(graph, edge.source()),
                make_target(graph, edge.target()),
            ))?;
        }
    }
    for node in graph0.nodes() {
        // TODO: `node.1` is a hack.
        let command = get_build_command(graph, *node.1);
        if let Some(command) = command {
            output_file.write_fmt(format_args!("{}:\n\t{}\n\n", make_target(graph, *node.1), command))?;
        }
        if let Import::Canister(canister_name) = node.0 {
            output_file.write_fmt(format_args!("\ndeploy@{}: canister@{}\n", canister_name, canister_name))?;
            output_file.write_fmt(format_args!(
                "\tdfx deploy --no-compile --network $(NETWORK) $(DEPLOY_FLAGS) $(DEPLOY_FLAGS.{}) {}\n\n", canister_name, canister_name
            ))?;
            // If the canister is assets, add `generate@` dependencies.
            let canister = pool.get_first_canister_with_name(&canister_name).unwrap(); // TODO: `unwrap`
            if canister.as_ref().get_info().is_assets() {
                let deps = canister.as_ref().get_info().get_dependencies();
                if !deps.is_empty() {
                    output_file.write_fmt(format_args!(
                        "\ncanister@{}: {}\n",
                        canister_name,
                        deps.iter().map(|name| format!("generate@{}", name)).join(" "),
                    ))?;
                }
            }
        }
    }

    Ok(())
}

fn make_target(graph: &Graph<Import, ()>, node_id: <Graph<Import, ()> as GraphBase>::NodeId) -> String {
    let node_value = graph.node_weight(node_id).unwrap();
    match node_value {
        Import::Canister(canister_name) => {
            // duplicate code
            let path1 = format!("$(ROOT_DIR)/.dfx/local/canisters/{}/{}.wasm", canister_name, canister_name);
            let path2 = format!("$(ROOT_DIR)/.dfx/local/canisters/{}/{}.did", canister_name, canister_name);
            format!("{} {}", path1, path2)
        }
        Import::FullPath(path) => format!("$(ROOT_DIR)/{}", path.to_str().unwrap().to_owned()), // FIXME: `unwrap`
        Import::Ic(principal_str) => format!("ic:{}", principal_str),
        Import::Lib(_path) => "".to_string(),
    }
}

fn get_build_command(graph: &Graph<Import, ()>, node_id: <Graph<Import, ()> as GraphBase>::NodeId) -> Option<String> {
    let node_value = graph.node_weight(node_id).unwrap();
    match node_value {
        Import::Canister(canister_name) =>
            Some(format!("dfx canister create {}\n\tdfx build --no-deps {}", canister_name, canister_name)),
        Import::FullPath(_path) => None,
        Import::Ic(principal_str) => Some(format!("dfx deploy --no-compile {}", principal_str)), // FIXME
        Import::Lib(_path) => None,
    }
}