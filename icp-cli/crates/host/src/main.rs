mod bindings;
mod cli;
mod command_config;
mod commands;
mod execute;
mod host;
mod nodes;
mod parse;
mod plan;
mod prettify;
mod registry;
mod tests;

use crate::cli::tree::CommandTree;
use crate::commands::identity;
use crate::nodes::node_descriptors;
use crate::parse::workflow::WorkflowModel;
use crate::registry::node_type_registry::NodeTypeRegistry;

extern crate command_descriptor_derive;
use crate::cli::descriptor::{CommandDescriptor, Dispatch};
use crate::cli::error::{CliError, CliResult};
use command_descriptor_derive::command_descriptor;

// fn x() {
//     let y : Vec<_> = vec![
//         crate::commands::identity::new,
//
//     ];
// }
fn builtin_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        identity::new::descriptor(),
        // Add other command descriptors here
    ]
}

const SIMPLE_WORKFLOW: &str = r#"
workflow:
    const:
        value: Hello, test!
    prettify:
        type: prettify
        inputs:
            input: const
    print:
        inputs:
            input: prettify
    print2:
        type: print
        inputs:
            input: prettify
"#;

fn workflow_command_descriptor(path: String, workflow: &str) -> CommandDescriptor {
    let path = path
        .split(" ")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let name = path.last().unwrap();
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str()) as &str;

    let subcommand = clap::Command::new(leaked).about("Run a workflow");
    let dispatch = Dispatch::Workflow(workflow.to_string());
    CommandDescriptor {
        path,
        subcommand,
        dispatch,
    }
}

fn workflow_descriptors() -> Vec<CommandDescriptor> {
    vec![
        workflow_command_descriptor("workflow".to_string(), SIMPLE_WORKFLOW),
        // Add workflow descriptors here
    ]
}

fn all_command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = builtin_command_descriptors();
    descriptors.extend(workflow_descriptors());
    descriptors
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let commands = all_command_descriptors();
    let command_tree = CommandTree::from_descriptors(commands);
    let command = command_tree.build_clap_command("icp");
    let matches = command.get_matches();

    let (descriptor, matches) = command_tree.get_descriptor(&matches);

    let r = match &descriptor.dispatch {
        Dispatch::Function(f) => f(&matches),
        Dispatch::Workflow(workflow) => execute_workflow(workflow).await,
    };
    if let Err(e) = r {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn execute_workflow(workflow: &str) -> CliResult {
    let mut registry = NodeTypeRegistry::new();
    registry.register(node_descriptors());

    let graph = WorkflowModel::from_string(workflow)
        .into_plan()
        .into_graph(&registry);

    graph
        .run()
        .await
        .map_err(|e| CliError(format!("Error executing workflow: {}", e)))
}
