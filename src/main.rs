use std::vec::IntoIter;
use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use rustyline::Editor;
use users::{get_user_by_uid, get_current_uid};
use colored::Colorize;
use std::fs;
use os_pipe::pipe;
use dirs::home_dir;
use rustyline::error::ReadlineError;
mod autocomplete;
use autocomplete::FileCompleter;
mod commons;
use commons::{read_commons,write_commons,get_home_usr};
use std::collections::HashMap;
mod aliases;
use aliases::{read_aliases,write_aliases};
use strip_ansi_escapes::strip_str;
pub fn r_pad(s: String,padded_to : usize) -> String {
    let len = padded_to - strip_str(&s).len();
    s + &"                                                                                           "[0..len]  // if you need more padding, idc
}

pub fn r_pad_array(vec: &[String],padded_to: usize) -> Vec<String>{
    vec.iter().map(|s| r_pad(s.to_string(),padded_to)).collect()
}
pub fn find_longest(strs :&[String]) -> usize {
    let mut longest = 0;
    for str in strs{
        if strip_str(str).len() > longest{
            longest = strip_str(str).len();
        }
    }
    longest
}
pub fn split_with_delimiter(s: &str, delimiter: &str) -> Vec<String> {
    let mut result:Vec<String> = Vec::new();
    if !s.contains(delimiter){
        result.push(s.to_string());
        return result;
    }
    let mut start = 0;
    let mut in_quotes = false;
    let mut i = 0;
    
    while i <= s.len() - delimiter.len() {
        if s[i..].starts_with(delimiter) && !in_quotes {
            result.push(s[start..i].to_string());
            i += delimiter.len();
            start = i;
        } else {
            if s[i..].starts_with('"') {
                in_quotes = !in_quotes;
            }
            i += 1;
        }
    }
    
    result.push(s[start..].to_string()); // Push the remaining part
    result
}
fn split_args(command : &str) -> (&str, IntoIter<String>){
    if let Some((command_,args_)) = command.split_once(' '){
        let mut result_args: Vec<String> = Vec::new();
        let mut in_quotes = false;
        let mut current_arg = String::new();

        for char in args_.chars() {
            match char {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ' ' => {
                    if !in_quotes {
                        if !current_arg.is_empty(){
                            result_args.push(current_arg.clone());
                            current_arg.clear();
                        }
                    }
                    else {
                        current_arg.push(char);
                    }
                }
                _ => {
                    current_arg.push(char);
                }
            }
        }
        if !current_arg.is_empty() {
            result_args.push(current_arg);
        }

        return (command_,result_args.into_iter());
    }
    else {return (command,Vec::new().into_iter());}

}

fn main_shell() -> bool{
    let user = get_user_by_uid(get_current_uid()).unwrap();
    let mut dir : String;
    
    let home_usr = get_home_usr();
    let hist_file =  home_usr.clone() + "/brick_shell/brick_shell_history.txt";
    let alias_file = home_usr.clone() + "/brick_shell/brick_shell_aliases.txt";

    let mut aliases = read_aliases(alias_file.clone());
    let mut perm_aliases = HashMap::<String,bool>::new();
    let mut alias_vec: Vec<String> = Vec::new();
    for key in aliases.keys(){
        perm_aliases.insert(key.to_string(), true);
    }
    if !fs::exists(home_usr.clone() + "/brick_shell").unwrap(){ // create brick shel directory if it doesn't yet exist
        fs::create_dir(home_usr.clone() + "/brick_shell").unwrap();
    }
    
    let mut commons = read_commons(home_usr.clone());
    if commons.is_empty(){
        commons.push("cd".to_string());
        commons.push("ls".to_string());
        commons.push("exit".to_string());
        commons.push("restart".to_string());
        commons.push("clear".to_string());
        commons.push("clear-history".to_string());

    }
    let h = FileCompleter {};
    let mut rl = Editor::new().unwrap();
    rl.set_helper(Some(h));
    
    if rl.load_history(&hist_file).is_err(){
        println!("Starting new history...");
    }

    loop {
        dir = env::current_dir().unwrap().display().to_string();
        let prompt = format!("╔({})-[{}] \n╚═{} ",user.name().to_str().unwrap().green().bold(),dir.blue().bold(),">".bold());

        let line =  rl.readline(&prompt);

        let input;
        match line {
            Ok(line) => {
                input = line;
            },
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(_) => {
                continue;
            }
        };
            
        rl.add_history_entry(&input).unwrap();
        //let commands_chain = input.trim().split(" && ").peekable();
        let commands: Vec<String> = split_with_delimiter(&input.trim(), " && ");
        for c in commands {
            let split_iter:Vec<String> = split_with_delimiter(&c.trim(), " | ");
            let mut split_commands = split_iter.into_iter().peekable();
            
            let mut prev_command: Option<Child>  = None;

            while let Some(command) = alias_vec.pop().or_else(||split_commands.next())  {
                let (command,args) = split_args(command.trim());
                match command {

                    "cd" => {
                        let new_dir = args.peekable().peek().map_or(home_dir().unwrap().display().to_string() , |x| x.to_string());

                        let path = Path::new(&new_dir);

                        if let Err(e) = env::set_current_dir(&path){
                            eprintln!("{}",e);
                        }

                        prev_command = None;
                    },
                    "exit"  => {
                        rl.append_history(&hist_file).unwrap();
                        write_commons(home_usr, commons);
                        write_aliases(alias_file, &aliases.into_iter().filter(|(key,_)| perm_aliases.get(key).copied().unwrap_or(false)).collect());
                        return false;
                    },
                    "restart"  => { // not pretty but I dont wanna deal with borrowing command stuff
                        rl.append_history(&hist_file).unwrap();
                        write_commons(home_usr, commons);
                        write_aliases(alias_file, &aliases.into_iter().filter(|(key,_)| perm_aliases.get(key).copied().unwrap_or(false)).collect());
                        return true;
                    },
                    "clear-history" => {
                        if rl.clear_history().is_err() {
                            eprintln!("Failed to clear history");
                        }

                        fs::remove_file(&hist_file).unwrap();
                    }
                    "alias" => {
                        let mut args = args.peekable();

                        let arg = args.peek().map_or(String::new(), |x| x.to_string());
                        if !(arg == "-t" || arg == "-p" || arg == ""){
                            eprintln!("Invalid Argument! Can only use '-t' for temporary and '-p' for permanent aliases");
                            continue;
                        }
                        args.next();
                        let perm = arg == "-p";

                        let alias = args.next().unwrap_or_default();
                        if alias.is_empty() {
                            eprintln!("Expected alias");
                            continue;
                        }

                        let alias_content = args.next().unwrap_or_default();
                        if alias_content.is_empty() {
                            eprintln!("Expected what '{}' is supposed to be an alias for", alias);
                            continue;
                        }

                        aliases.insert(alias.clone(), alias_content);
                        perm_aliases.insert(alias, perm);

                    }
                    "ls" => {
                        let arg = args.peekable().peek().map_or(String::new(), |x| x.to_string());
                        if !(arg == "-e" || arg == "-a" || arg == ""){
                            eprintln!("Invalid Argument! Can only use '-e' for extended and '-a' for all-view");
                            continue;
                        }
                        let extended = arg == "-e";
                        let all = arg == "-a";
                        let files = fs::read_dir(&dir).unwrap();
                        let mut ls_output = String::new();
                        for file in files {
                            let file_type = file.as_ref().unwrap().file_type().unwrap();
                            let file_name = file.unwrap().file_name().into_string().unwrap();
                            let starts_with_dot = file_name.starts_with(".");
                            if file_type.is_dir() {
                                if starts_with_dot && (extended || all){
                                    ls_output.push_str(&format!("{}/\n",file_name.purple().bold()));
                                }else if !starts_with_dot{
                                    ls_output.push_str(&format!("{}/\n",file_name.blue().bold()));
                                }
                            }else if file_type.is_file() {
                                if !file_name.ends_with(".tmp") || all{

                                    if starts_with_dot && (extended || all){
                                        ls_output.push_str(&format!("{}\n",file_name));
                                    }
                                    else if !starts_with_dot && file_name.contains("."){
                                        ls_output.push_str(&format!("{}\n",file_name));
                                    }else if !starts_with_dot {
                                        ls_output.push_str(&format!("{}\n",file_name.green().bold()));
                                    }
                                }
                            }else if file_type.is_symlink() {
                                ls_output.push_str(&format!(">{}\n",file_name.bright_cyan()));
                            }
                        }
                        // No pipe -> print
                        if split_commands.peek().is_none() {
                            let n_items = ls_output.lines().count();
                            if n_items > 10 {
                                // multiple cols
                                let mut items: Vec<String> = ls_output.split("\n").map(|s| s.to_string()).collect();
                                let mut longest_len = 0;
                                for item in &items{
                                    if strip_str(&item).len() > longest_len{
                                        longest_len = strip_str(&item).len();
                                    }
                                }
                                let cols = if n_items / 10 < 5 {n_items / 10} else {5}; 
                                
                                let rows = (n_items / cols) + 1;
                                
                                let len = items.len();

                                for _ in len..(rows * cols) {
                                    items.push(String::new()); // prevent accessing non-exsiting strings
                                }
                                let mut padded_cols: Vec<Vec<String>> = Vec::new();
                                
                                for col in 0..cols {
                                    let col_x_rows = col * rows;
                                    let pad = find_longest(&items[(col_x_rows)..(col_x_rows + rows) ]);
                                    padded_cols.push(r_pad_array(&items[(col_x_rows)..(col_x_rows + rows) ], pad + 3));
                                }

                                for i in 0..rows{
                                    let mut line = String::new();
                                    for col in &padded_cols{
                                        line.push_str(&col[i]);
                                    }
                                    println!("{}",line);
                                }

                            }else{
                                print!("{}",ls_output);
                            }
                        }else{
                            let (reader,mut writer) = pipe().expect("Pipe failed");
                            writer.write_all(ls_output.as_bytes()).unwrap();

                            drop(writer);

                            prev_command = Some(
                                Command::new("cat") 
                                .stdin(Stdio::from(reader))
                                .stdout(Stdio::piped())
                                .spawn()
                                .expect("Failed to spawn command")
                            );


                        }
                    },
                    command => {

                        let stdin = prev_command
                        .map_or(
                                Stdio::inherit(),
                                |output: Child| Stdio::from(output.stdout.unwrap())
                            );
                        let stdout = if split_commands.peek().is_some() {
                            Stdio::piped()
                            } else {
                                Stdio::inherit()
                            };

                            let output = Command::new(command)
                            .args(args)
                            .stdin(stdin)
                        .stdout(stdout)
                        .spawn();
                        
                    match output {
                        Ok(output) => {prev_command = Some(output); if !commons.contains(&command.to_string()) {commons.push(command.to_string() )};}, // Only push to commons if true command 
                        Err(_e) => {
                            prev_command = None; 
                            if aliases.contains_key(command){
                                if !commons.contains(&command.to_string()) {commons.push(command.to_string());} // push custom commands
                                let cmd = aliases.get(&command.to_string()).unwrap();
                                let cmds = split_with_delimiter(&cmd.trim(), " && ");
                                for c in cmds.iter().rev(){
                                    alias_vec.push(c.to_string());
                                }
                                continue;
                            }else{

                                if command.to_string() != "" {
                                    eprintln!("Command '{}' was not found!",command.to_string())};
                                }
                            }
                        }
                    }
                }
            }
            if let Some(mut final_cmd) = prev_command {
                final_cmd.wait().unwrap();
            }
        }   
    }

    
}


fn main() {
    env::set_var("SHELL", env::current_exe().unwrap());
    if env::args().nth(1).is_some() {
        let arg = env::args().nth(1).unwrap();
        if arg == "-h" || arg == "--help" {
            println!("Usage: brick_shell [options]");
            println!("Options:");
            println!("  -v, --version Show version information");
            println!("  -h, --help    Show this help message");
            return;
        }
        if arg == "-v" || arg == "--version" {
            println!("v0.1");
            return;
        }
    }
    // Check if this shell was launched as a login shell
    let is_login_shell = env::args()
        .nth(0)
        .map(|arg| arg.starts_with('-'))
        .unwrap_or(false);


    // If it's a login shell, source system-wide and user profile scripts
    if is_login_shell {
        Command::new("sh")
            .arg("-c")
            .arg("source /etc/profile")
            .status()
            .ok();

        if let Some(home) = home_dir() {
            let user_profile = format!("{}/.profile", home.display());
            if Path::new(&user_profile).exists() {
                Command::new("sh")
                    .arg("-c")
                    .arg(format!("source {}", user_profile))
                    .status()
                    .ok();
            }
        }
    }


    while main_shell() {
        println!("Restarted successfully");
    }
    return;
}
