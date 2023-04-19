use super::ExtensionManager;
use colored::Colorize;

use crate::lib::{
    error::ExtensionError,
    extension::{manifest::ExtensionManifest, Extension},
};

impl ExtensionManager {
    pub fn list_installed_extensions(&self) -> Result<Vec<Extension>, ExtensionError> {
        let dir_content = dfx_core::fs::read_dir(&self.dir)
            .map_err(ExtensionError::ExtensionsDirectoryIsNotReadable)?;

        Ok(dir_content
            .filter_map(|v| {
                let dir_entry = v.ok()?;
                if dir_entry.file_type().map_or(false, |e| e.is_dir())
                    && !dir_entry.file_name().to_str()?.starts_with(".tmp")
                {
                    Some(Extension::from(dir_entry))
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn dfx_help_print_installed_extensions(&self) -> String {
        pub fn calculate_padding(longest_extension_name: usize) -> usize {
            const LONGEST_DFX_COMMAND: usize = 10; // "dfx quickstart"
            const PADDING: usize = 4;
            if longest_extension_name < LONGEST_DFX_COMMAND + 1 {
                LONGEST_DFX_COMMAND + PADDING
            } else {
                LONGEST_DFX_COMMAND - longest_extension_name + PADDING
            }
        }
        // FIXME
        pub fn wrap_text(text: &str, padding: usize) -> String {
            const PREFIX_PADDING: usize = 4;
            const MAX_LINE_LENGTH: usize = 80;
            const INDENT: usize = 4;
            // first line should not be indented
            // subsequent lines should be indented
            let mut lines = text
                .split_whitespace()
                .fold((String::new(), 0), |(mut lines, mut line_length), word| {
                    if line_length + word.len() + 1 > MAX_LINE_LENGTH {
                        lines.push_str(&" ".repeat(INDENT));
                        line_length = INDENT;
                    }
                    if line_length > 0 {
                        lines.push(' ');
                        line_length += 1;
                    }
                    lines.push_str(word);
                    line_length += word.len();
                    (lines, line_length)
                })
                .0
                .lines()
                .map(|line| format!("{}{}", " ".repeat(PREFIX_PADDING), line))
                .collect::<Vec<_>>();
            lines.insert(0, " ".repeat(padding));
            lines.join("")
        }
        let mut extensions = self.list_installed_extensions().unwrap_or_default();
        if extensions.is_empty() {
            return "No extensions installed.".to_string();
        }
        let mut output = String::from("EXTENSIONS:\n").yellow().to_string();
        extensions.sort_by(|a, b| a.name.cmp(&b.name));
        let longest_ext_name = extensions.iter().map(|v| v.name.len()).max().unwrap(); // ok to unwrap, we already checked that extensions is not empty
        let padding = calculate_padding(longest_ext_name);
        for extension in extensions {
            let extension_manifest = ExtensionManifest::from_extension_directory(
                self.get_extension_directory(&extension.name),
            )
            .unwrap(); // TODO
            output.push_str(&format!(
                "    {name:padding$}{desc}",
                name = &extension.name.green(),
                desc = wrap_text(&extension_manifest.description.unwrap(), padding), // TODO
                padding = padding
            ));
        }
        output
    }
}
