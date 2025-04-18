use crate::cli::descriptor::{CommandDescriptor, Dispatch};
use crate::cli::error::{CliError, CliResult};
use clap::ArgMatches;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CommandTree {
    descriptor: Option<CommandDescriptor>,  // leaf commands only
    children: HashMap<String, CommandTree>, // subcommands
}

impl CommandTree {
    fn new() -> Self {
        Self {
            descriptor: None,
            children: HashMap::new(),
        }
    }
    pub fn from_descriptors(descriptors: Vec<CommandDescriptor>) -> Self {
        let mut root = CommandTree::new();

        for desc in descriptors {
            let mut current = &mut root;

            for part in &desc.path {
                current = current
                    .children
                    .entry(part.clone())
                    .or_insert_with(CommandTree::new);
            }

            // Attach the actual subcommand at the leaf
            current.descriptor = Some(desc);
        }

        root
    }

    pub fn build_clap_command(&self, name: &str) -> clap::Command {
        let leaked: &'static str = Box::leak(name.to_string().into_boxed_str()) as &str;
        let mut cmd = if let Some(desc) = &self.descriptor {
            desc.subcommand.clone().name(leaked)
        } else {
            clap::Command::new(leaked)
        };

        // Add subcommands
        for (child_name, child_node) in &self.children {
            let subcommand = child_node.build_clap_command(child_name);
            cmd = cmd.subcommand(subcommand);
        }

        cmd
    }

    pub fn get_descriptor<'a, 'b>(
        &'a self,
        matches: &'b ArgMatches,
    ) -> Result<(&'a CommandDescriptor, &'b ArgMatches), CliError> {
        match (matches.subcommand(), self.descriptor.as_ref()) {
            (None, Some(desc)) => Ok((desc, matches)),
            (Some((subcommand, sub_matches)), _) => match self.children.get(subcommand) {
                Some(child) => child.get_descriptor(sub_matches),
                None => Err(CliError(format!("Unknown subcommand: {}", subcommand))),
            },
            (None, None) => Err(CliError("No command descriptor at this node".into())),
        }
    }

    pub(crate) fn dispatch(&self, matches: &ArgMatches) -> CliResult {
        match matches.subcommand() {
            Some((subcommand, sub_matches)) => match self.children.get(subcommand) {
                Some(child) => child.dispatch(sub_matches),
                None => Err(CliError(format!("Unknown subcommand: {}", subcommand))),
            },
            None => match &self.descriptor {
                Some(desc) => match &desc.dispatch {
                    Dispatch::Function(f) => f(matches),
                    Dispatch::Workflow(workflow) => {
                        todo!()
                    } // more dispatch variants
                },
                None => Err(CliError("No command to dispatch at this node".into())),
            },
        }
    }
}
