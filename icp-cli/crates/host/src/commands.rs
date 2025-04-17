use crate::commands::identity::new::CommandDescriptor;

pub mod identity;

pub fn commands() -> Vec<CommandDescriptor> {
    vec![identity::new::descriptor()]
}
