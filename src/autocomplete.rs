use rustyline::completion::Completer;
use rustyline::validate::ValidationContext;
use rustyline::validate::ValidationResult;
use rustyline::Context;
use rustyline::Helper;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use std::borrow::Cow;
use std::env;
use std::fs;

use crate::commons;
use commons::{read_commons,get_home_usr};
pub struct FileCompleter;

fn get_dir_and_input(line: &str, pos: usize) -> (usize,String,String,String){
    let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
    let mut input = &line[start..pos];
    let mut first = String::new();
    // Get the current directory
    let mut dir = env::current_dir().unwrap().display().to_string();
    if input.contains("/"){
        let (first_part,second_part) = input.split_at(input.rfind('/').unwrap());
        input = &second_part[1..];
        first = first_part.to_owned() + "/";
        dir = dir + "/" + &first;
    }

    (start,input.to_string(),dir,first)
}

fn hint_dir_item(line: &str, pos: usize, _ctx: &Context) -> Option<String>{
    // Find the position where the filename starts
    let (_,input, dir,_) = get_dir_and_input(line, pos);
    // Get matching file names in the target directory
    match fs::read_dir(&dir) {
        Ok(entries) => 
            for entry in entries{

                if entry.is_ok() {
                    let e = entry.unwrap();
                    let mut file_name = e.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(&input) && !file_name.ends_with(".tmp") && !file_name.starts_with(".") && input != ""{
                        file_name = file_name[file_name.find(&input).unwrap() + input.len()..].to_string();
                        if e.file_type().unwrap().is_dir(){
                            return Some(file_name + "/");
                        }else{
                            return Some(file_name);
                        }
                    } 
                }
            }
            
            
        Err(_) => return None
    };

    return None;
}

fn complete_dir_item(line: &str, pos: usize, _ctx: &Context) -> (usize,Vec<String>){
    let (start,input,dir,first) = get_dir_and_input(line, pos);
    // Get matching file names in the target directory
    let files = match fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let file_name = e.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(&input[..]) {
                        if e.file_type().unwrap().is_dir(){
                            Some(first.to_owned() + &file_name + "/")
                        }else{
                            Some(first.to_owned() + &file_name)
                        }
                    } else {
                        None
                    }
                })
            })
            .collect(),
        Err(_) => vec![],
    };
    (start,files)
}
impl Completer for FileCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let (start,files) = complete_dir_item(line, pos, _ctx);

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
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        const LIGHT_GRAY: &str = "\x1b[37m";
        const RESET: &str = "\x1b[0m";

        // Apply light gray highlighting to the hint
        Cow::Owned(format!("{}{}{}", LIGHT_GRAY, hint, RESET))
    }
}

impl Hinter for FileCompleter {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, _ctx: &Context) -> Option<String> {
        if line != "" && (line.find(" ").is_none() || (line.rfind(" | ").is_some() && line.rfind(" ").unwrap() == pos)) {
            let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
            let input = &line[start..pos];
            let commons = read_commons(get_home_usr());

            for common in commons{
                if common.starts_with(&input){
                    return Some(common[common.find(&input).unwrap() + input.len()..].to_string());
                }
            }
            None
        }else{
            hint_dir_item(line, pos, _ctx)
        }
    }
}

impl Helper for FileCompleter {}
