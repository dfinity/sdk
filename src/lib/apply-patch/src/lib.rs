//! Applies patches in the [Unified Format](https://www.gnu.org/software/diffutils/manual/html_node/Detailed-Unified.html) to files.

// This will eventually be its own package with more fleshed-out features, but currently just has the bare minimum to implement `dfx new` against it.
// Missing spots are marked `todo:` with a corresponding `mvp:` section explaining why it doesn't need to be in yet.

// todo: reimplement ::patch. benefits: non-borrowed error, binary patching, consistent newlining, validation of non-overlapping and sortedness.
// mvp: borrowed errors can be formatted into anyhow, all our patches are text patches to text files in a reasonable format
use patch::{Line, Patch};
use thiserror::Error;

/// Applies a single-file patch to `content`.
///
/// File paths in the patch file are ignored. Equivalent to `Settings::default().apply_to(patch, content)`.
pub fn apply_to(patch: &Patch, content: &str) -> Result<String, MismatchError> {
    Settings::new().apply_to(patch, content)
}

/// Settings for patch application.
#[derive(Debug, Clone)]
pub struct Settings {
    ignore_line_numbers: bool,
    ignore_whitespace: bool,
    // todo: implement multi-patch application to directory. mvp: our patch files are specific
    _reject_relative_path_segments: bool,
}

impl Settings {
    /// Initializes the default settings:
    ///
    /// * The line numbers listed in the patch must be the location in the content
    /// * A whitespace-only difference in the content does not invalidate the patch
    /// * File paths in multi-file patches are disallowed from containing `..`
    pub fn new() -> Self {
        Self {
            ignore_line_numbers: false,
            ignore_whitespace: true,
            _reject_relative_path_segments: true,
        }
    }
    /// Allows line numbers to differ between the patch file and the content.
    pub fn ignore_line_numbers(self) -> Self {
        Self {
            ignore_line_numbers: true,
            ..self
        }
    }
    /// Requires whitespace to be an exact match between the patch file's context/deleted lines and the content.
    pub fn exact_whitespace(self) -> Self {
        Self {
            ignore_whitespace: false,
            ..self
        }
    }
    /// Allows `..` in file paths in multi-file patches (not recommended).
    pub fn allow_relative_path_segments(self) -> Self {
        Self {
            _reject_relative_path_segments: false,
            ..self
        }
    }
    /// Applies a single-file patch to `content`. File paths in the patch file are ignored.
    // todo: use an iterator instead of returning String. mvp: our files are small.
    pub fn apply_to(&self, patch: &Patch, content: &str) -> Result<String, MismatchError> {
        let mut expected_lines = vec![];
        let mut prev_idx = 0;
        let mut new_content = String::new();
        for hunk in &patch.hunks {
            // first, assemble the list of lines we expect to find in `content`
            expected_lines.clear();
            expected_lines.reserve(hunk.lines.len());
            for hunk_line in &hunk.lines {
                if let &Line::Context(s) | &Line::Remove(s) = hunk_line {
                    expected_lines.push(s);
                }
            }
            // second, find and attempt to match them
            let found_range = if self.ignore_line_numbers {
                // todo: implement line-number-agnostic patching. mvp: all our patch files have exact line numbers known.
                unimplemented!()
            } else {
                // todo: sort patch contents so the line cursor can be cached. mvp: all our patches are short and single-range.
                let mut lines = content.match_indices('\n');
                // line numbers are all 1-indexed - this is specifically the parsed number
                let start_line = hunk.old_range.start as usize - 1;
                // line n starts one past the n-1th newline
                let start = if start_line == 0 {
                    0
                } else {
                    lines
                        .nth(start_line - 1)
                        .ok_or(MismatchError::NotEnoughLines)?
                        .0
                        + 1
                };
                // start..end should be the byte range in `content` to be patched
                let end = lines
                    .nth(expected_lines.len() - 1)
                    .ok_or(MismatchError::NotEnoughLines)?
                    .0
                    + 1;
                if compare_range(
                    &content[start..end],
                    &expected_lines,
                    self.ignore_whitespace,
                ) {
                    start..end
                } else {
                    return Err(MismatchError::LineMismatch);
                }
            };
            // first copy to the output all the content between either the last patched range or the beginning of the file, and the beginning of this patch
            new_content.push_str(&content[prev_idx..found_range.start]);
            prev_idx = found_range.end;
            // then interleave the context lines with the added lines
            let mut orig_lines = content[found_range].lines();
            for hunk_line in &hunk.lines {
                match hunk_line {
                    &Line::Context(_) => {
                        // in the case of a context line, push the original line, not the one from the patch file
                        // this may be a whitespace-insensitive patch, and we don't want to modify any lines that aren't marked `-`
                        new_content.push_str(orig_lines.next().unwrap());
                        new_content.push('\n');
                    }
                    &Line::Add(s) => {
                        new_content.push_str(s);
                        new_content.push('\n');
                    }
                    &Line::Remove(_) => {
                        orig_lines.next().unwrap();
                    }
                }
            }
        }
        // finally, copy everything between the final patch and the end of the file
        new_content.push_str(&content[prev_idx..]);
        Ok(new_content)
    }
}

fn compare_range(content: &str, lines: &[&str], ignore_whitespace: bool) -> bool {
    for (left, &right) in content.lines().zip(lines) {
        if ignore_whitespace {
            if left != right {
                return false;
            }
        } else {
            if !left
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .eq(right.chars().filter(|ch| !ch.is_whitespace()))
            {
                return false;
            }
        }
    }
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum MismatchError {
    #[error("File did not contain enough lines")]
    NotEnoughLines,
    #[error("Mismatch between context/removal line and file")]
    LineMismatch,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Mismatch(#[from] MismatchError),
}
