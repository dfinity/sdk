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
use command_descriptor_derive::command_descriptor;

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

    // command_tree.dispatch(&matches).unwrap_or_else(|e| {
    //     eprintln!("Error: {}", e);
    //     std::process::exit(1);
    // });

    let (descriptor, matches) = match command_tree.get_descriptor(&matches) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match &descriptor.dispatch {
        Dispatch::Function(f) => {
            if let Err(e) = f(&matches) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Dispatch::Workflow(workflow) => {
            if let Err(e) = execute_workflow(workflow).await {
                eprintln!("Workflow error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
async fn execute_workflow(workflow: &str) -> Result<(), String> {
    let mut registry = NodeTypeRegistry::new();
    registry.register(node_descriptors());

    // let model = WorkflowModel::from_string(workflow);
    // let plan = WorkflowPlan::from_model(model);
    // let graph = ExecutionGraph::from_plan(plan, &registry);

    let graph = WorkflowModel::from_string(workflow)
        .into_plan()
        .into_graph(&registry);

    let result = graph.run().await;
    if let Err(e) = result {
        println!("Error executing workflow: {}", e);
        std::process::exit(1);
    }
    Ok(())
}
