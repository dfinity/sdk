mod cli;
mod commands;
mod host;
mod project;
mod tests;
mod workflow;

use crate::cli::tree::CommandTree;
use crate::commands::identity;
use std::collections::HashMap;
use workflow::nodes::node_descriptors;
use workflow::parse::workflow::WorkflowModel;
use workflow::registry::node_type_registry::NodeTypeRegistry;

extern crate command_descriptor_derive;
use crate::cli::descriptor::{CommandDescriptor, Dispatch};
use crate::cli::error::{CliError, CliResult};
use command_descriptor_derive::command_descriptor;

fn builtin_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        identity::new::descriptor(),
        // Add other command descriptors here
    ]
}

const SIMPLE_WORKFLOW: &str = r#"
workflow:
  const-string:
    inputs:
      value: Hello, test!
  prettify:
    inputs:
      input:
        node: const-string
  print:
    inputs:
      input:
        node: prettify
  print2:
    type: print
    inputs:
      input:
        node: prettify
"#;

const BUILD_WORKFLOW: &str = r#"
parameters:
  canister-name:
    kind: string
  builder:
    kind: node-type

workflow:
  any-builder:
    type:
      parameter: builder
    inputs:
      package:
        parameter: canister-name
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
        workflow_command_descriptor("simple".to_string(), SIMPLE_WORKFLOW),
        workflow_command_descriptor("build".to_string(), BUILD_WORKFLOW),
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

    let parameters = HashMap::from([
        (
            "canister-name".to_string(),
            "svelte-rust-backend".to_string(),
        ),
        ("builder".to_string(), "rust-builder".to_string()),
    ]);

    let graph = WorkflowModel::from_string(workflow)
        .into_plan(parameters, &registry)
        .into_graph(&registry)
        .map_err(|e| {
            CliError(format!(
                "Error creating execution graph from workflow: {}",
                e
            ))
        })?;

    graph
        .run()
        .await
        .map_err(|e| CliError(format!("Error executing workflow: {}", e)))
}
