use std::fs::File;
use std::io::Write;

use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::builders::CanisterBuilder;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::{CanisterPool, Import};
use crate::lib::builders::custom::CustomBuilder;
use clap::Parser;
use petgraph::graph::DiGraph;
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

pub fn exec(env1: &dyn Environment, opts: RulesOpts) -> DfxResult {
    let env = create_anonymous_agent_environment(env1, None)?;
    // let log = env.get_logger();

    // Read the config.
    let config = env.get_config_or_anyhow()?;

    let mut output_file: Box<dyn Write> = match opts.output {
        Some(filename) => Box::new(File::open(filename)?),
        None => Box::new(std::io::stdout()),
    };

    match &config.get_config().canisters {
        Some(canisters) => {
            output_file.write_fmt(format_args!(".PHONY:"))?;
            for canister in canisters {
                output_file.write_fmt(format_args!(" canister:{}", canister.0))?;
            };
            output_file.write_fmt(format_args!("\n\n"))?;
            for canister in canisters {
                // duplicate code
                let path1 = format!(".dfx/local/canisters/{}/{}.wasm", canister.0, canister.0);
                let path2 = format!(".dfx/local/canisters/{}/{}.did", canister.0, canister.0);
                output_file.write_fmt(format_args!("canister:{}: \\\n  {} {}\n\n", canister.0, path1, path2))?;
            };
        }
        None => {}
    };

    let env = create_anonymous_agent_environment(env1, None)?;

    // Load dependencies for Make rules:
    let builder = CustomBuilder::new(env1)?; // TODO: hack // TODO: `&env` instead?
    // TODO: hack:
    let canister_names = config.get_config().canisters.as_ref().unwrap().keys().map(|k| k.to_string()).collect::<Vec<String>>();
    let pool = CanisterPool::load(
        &env, // if `env1`,  fails with "NetworkDescriptor only available from an AgentEnvironment"
        false,
        &canister_names, // FIXME: `unwrap`
    )?;
    builder.read_all_dependencies(
        &env,
        &pool,
        env.get_cache().as_ref(),
    )?;

    let graph0 = env.get_imports().borrow();
    let graph = graph0.graph();
    for edge in graph.edge_references() {
        let target_value = graph.node_weight(edge.target()).unwrap();
        if let Import::Lib(_) = target_value { // TODO: Unused, because package manager never update existing files (but create new dirs)
            output_file.write_fmt(format_args!(
                "{}\n",
                make_target(graph, edge.source())))?;
        } else {
            output_file.write_fmt(format_args!(
                "{}: {}\n",
                make_target(graph, edge.source()),
                make_target(graph, edge.target())))?;
        }
        let command = get_build_command(graph, edge.source());
        if let Some(command) = command {
            output_file.write_fmt(format_args!("\t{}\n\n", command))?;
        }
    }

    Ok(())
}

fn make_target(graph: &Graph<Import, ()>, node_id: <Graph<Import, ()> as GraphBase>::NodeId) -> String {
    let node_value = graph.node_weight(node_id).unwrap();
    match node_value {
        Import::Canister(canister_name) => {
            // duplicate code
            let path1 = format!(".dfx/local/canisters/{}/{}.wasm", canister_name, canister_name);
            let path2 = format!(".dfx/local/canisters/{}/{}.did", canister_name, canister_name);
            format!("{} {}", path1, path2)
        }
        Import::FullPath(path) => path.to_str().unwrap().to_owned(), // FIXME: `unwrap`
        Import::Ic(principal_str) => format!("ic:{}", principal_str),
        Import::Lib(path) => "".to_string(),
    }
}

fn get_build_command(graph: &Graph<Import, ()>, node_id: <Graph<Import, ()> as GraphBase>::NodeId) -> Option<String> {
    let node_value = graph.node_weight(node_id).unwrap();
    match node_value {
        Import::Canister(canister_name) => Some(format!("dfx build --no-deps {}", canister_name)),
        Import::FullPath(path) => None,
        Import::Ic(principal_str) => Some(format!("dfx deploy --no-compile {}", principal_str)), // FIXME
        Import::Lib(path) => None,
    }
}