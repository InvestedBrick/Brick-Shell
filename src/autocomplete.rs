use rustyline::completion::Completer;
use rustyline::validate::ValidationContext;
use rustyline::validate::ValidationResult;
use rustyline::Context;
use rustyline::Helper;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use std::borrow::Cow::{self, Borrowed};
use std::env;
use std::fs;

pub struct FileCompleter;

impl Completer for FileCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // Find the position where the filename starts
        let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let input = &line[start..pos];

        // Get the current directory
        let dir = env::current_dir().unwrap().display().to_string();

        // Get matching file names in the current directory
        let files = match fs::read_dir(dir) {
            Ok(entries) => entries
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        let file_name = e.file_name().to_string_lossy().to_string();
                        if file_name.starts_with(input) {
                            Some(file_name)
                        } else {
                            None
                        }
                    })
                })
                .collect(),
            Err(_) => vec![],
        };

        // Return completion candidates
        Ok((start, files))
    }
}

impl Validator for FileCompleter {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        // No validation needed, just return Ok
        Ok(ValidationResult::Valid(None))
    }
}

impl Highlighter for FileCompleter {
    fn highlight<'a>(&self, line: &'a str, _pos: usize) -> Cow<'a,str> {
        // No highlighting needed, just return an empty match
        Borrowed(line)
    }
}

impl Hinter for FileCompleter {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context) -> Option<String> {
        // No hints needed, just return None
        None
    }
}

impl Helper for FileCompleter {}
