use super::ExtensionManager;
use crate::lib::{
    error::ExtensionError,
    extension::{manifest::ExtensionManifest, Extension},
};

use console::style;
use textwrap::{termwidth, wrap, Options};

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
        // explenation for const:
        // SUBCOMMANDS:
        //     quickstart    Use the command to
        // ^^#^          ^^$^    perform action
        //                   ^^%^
        // <-------- CLAP_HELP_WIDTH --------->
        // <-------- TRUE TERMINAL WINDOW WIDTH ------->
        const INDENT_BEFORE_COMMAND: usize = 4; // ^^#^
        const LONGEST_DFX_COMMAND: usize = 10; // "quickstart".len()
        const PADDING_BETWEEN_COMMAND_AND_SUMMARY: usize = 4; // ^^$^
        const INDENT_COMMAND_SUMMARY: usize = 4; // ^^%^
        const CLAP_HELP_WIDTH: usize = 100; // clap sets a default width for printing
        fn wrap_text(text: &str, len_longest_ext: usize, len_current_ext: usize) -> String {
            let termwidth = std::cmp::min(termwidth(), CLAP_HELP_WIDTH);
            let ext_name_offset = std::cmp::max(len_longest_ext, LONGEST_DFX_COMMAND);
            let padding = ext_name_offset - len_current_ext + PADDING_BETWEEN_COMMAND_AND_SUMMARY;
            let command_name_with_margins = INDENT_BEFORE_COMMAND + len_current_ext + padding;

            // the width for the column of text for the extension summary
            let wraptext_width = termwidth.saturating_sub(command_name_with_margins);
            // when summary for the command doesn't fit one line, each next line should be indented by X spaces
            let subsequent_indent = " ".repeat(INDENT_COMMAND_SUMMARY);
            let options = Options::new(wraptext_width).subsequent_indent(&subsequent_indent);

            // subsequent_indent only pushes each new line by X spaces from them left, however,
            // this is insufficient because the new line should start directly under the previous one.
            // This would not be possible to achive only with Options.subsequent_indent,
            // because it modifies the width of the column of the text which is undesirable
            // (think of it as if increasing the value of right margin).
            let newline_indent = " ".repeat(command_name_with_margins);
            let wrapped_text = wrap(text, &options).join(&format!("\n{newline_indent}"));
            // The whitespace between extension name and and extension summary.
            // This would not be possible to achive with Options.initial_indent,
            // because it modifies the width of the column of the text which is undesirable
            // (think of it as if increasing the value of right margin).
            let initial_indent = " ".repeat(padding);
            format!("{initial_indent}{wrapped_text}",)
        }

        let mut extensions = self.list_installed_extensions().unwrap_or_default();
        if extensions.is_empty() {
            return "No extensions installed.".to_string();
        }

        let mut output = style(String::from("EXTENSIONS:\n")).yellow().to_string();

        // name of extension should not be longer than 30 chars
        // ok to unwrap, we already checked that extensions is not empty
        let len_longest_ext_name =
            std::cmp::min(30, extensions.iter().map(|v| v.name.len()).max().unwrap());

        extensions.sort_by(|a, b| a.name.cmp(&b.name));
        for extension in extensions {
            let extension_name = if extension.name.len() > 30 {
                // Name of ext will get cropped if its longer than 30 chars.
                // In such case, if the user wants to see full name of extension
                // they should issue `dfx extension list`
                format!("{}...", &extension.name[..27])
            } else {
                extension.name.to_string()
            };
            let desc = {
                let text = match ExtensionManifest::from_extension_directory(
                    self.get_extension_directory(&extension.name),
                ) {
                    Ok(extension_manifest) => extension_manifest.summary,
                    Err(err) => format!("Error while loading extension manifest: {err}"),
                };
                wrap_text(&text, len_longest_ext_name, extension_name.len())
            };
            let name = style(&extension_name).green();
            let initial_indent = " ".repeat(INDENT_BEFORE_COMMAND);
            output.push_str(&format!("{initial_indent}{name}{desc}\n",));
        }
        output
    }
}
