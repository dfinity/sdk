//! Applies patches in the [Unified Format](https://www.gnu.org/software/diffutils/manual/html_node/Detailed-Unified.html) to files.

// This will eventually be its own package with more fleshed-out features, but currently just has the bare minimum to implement `dfx new` against it.
// Missing spots are marked `todo:` with a corresponding `mvp:` section explaining why it doesn't need to be in yet.

use std::ops::Range;

// todo: reimplement ::patch. benefits: non-borrowed error, binary patching, consistent newlining, validation of non-overlapping and sortedness.
// mvp: borrowed errors can be formatted into anyhow, all our patches are text patches to text files in a reasonable format
use patch::{Hunk, Line, Patch};
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
        assert!(is_patch_coherent(patch));
        let original_content = content;
        let mut expected_lines = vec![];
        let mut patched_up_to = 0;
        let mut patched_content = String::new();
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
                self.find_fixed_range(original_content, hunk, &expected_lines)?
            };
            // first copy to the output all the content between either the last patched range or the beginning of the file, and the beginning of this patch
            patched_content.push_str(&original_content[patched_up_to..found_range.start]);
            patched_up_to = found_range.end;
            // then interleave the context lines with the added lines
            self.patch_content(&original_content[found_range], hunk, &mut patched_content);
        }
        // finally, copy everything between the final patch and the end of the file
        patched_content.push_str(&original_content[patched_up_to..]);
        Ok(patched_content)
    }

    fn patch_content(&self, original_content: &str, hunk: &Hunk, patched_content: &mut String) {
        let mut orig_lines = original_content.lines();
        for hunk_line in &hunk.lines {
            match *hunk_line {
                Line::Context(_) => {
                    // in the case of a context line, push the original line, not the one from the patch file
                    // this may be a whitespace-insensitive patch, and we don't want to modify any lines that aren't marked `-`
                    patched_content.push_str(orig_lines.next().unwrap());
                    patched_content.push('\n');
                }
                Line::Add(s) => {
                    patched_content.push_str(s);
                    patched_content.push('\n');
                }
                Line::Remove(_) => {
                    orig_lines.next().unwrap();
                }
            }
        }
    }

    fn find_fixed_range(
        &self,
        original_content: &str,
        hunk: &Hunk,
        expected_lines: &[&str],
    ) -> Result<Range<usize>, MismatchError> {
        let mut lines = original_content.match_indices('\n');
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
        if equal_range(
            &original_content[start..end],
            expected_lines,
            self.ignore_whitespace,
        ) {
            Ok(start..end)
        } else {
            return Err(MismatchError::LineMismatch);
        }
    }
}

fn is_patch_coherent(patch: &Patch) -> bool {
    // each patch file should be sorted and non-overlapping
    patch
        .hunks
        .iter()
        .zip(&patch.hunks[1..])
        .all(|(h1, h2)| h1.old_range.start + h1.old_range.count < h2.old_range.start)
}
fn equal_range(content: &str, lines: &[&str], ignore_whitespace: bool) -> bool {
    content.lines().zip(lines).all(|(left, &right)| {
        if ignore_whitespace {
            left.chars()
                .filter(|ch| !ch.is_whitespace())
                .eq(right.chars().filter(|ch| !ch.is_whitespace()))
        } else {
            left == right
        }
    })
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
