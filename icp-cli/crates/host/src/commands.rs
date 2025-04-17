pub mod identity;

use crate::cli::descriptor::CommandDescriptor;

pub fn commands() -> Vec<CommandDescriptor> {
    vec![identity::new::descriptor()]
}
