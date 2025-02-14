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
        let prompt = format!("({})[{}] > ",user.name().to_str().unwrap().green().bold(),dir.blue().bold());

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
            
        //stdin().read_line(&mut input).unwrap();
        rl.add_history_entry(&input).unwrap();
        let mut commands = input.trim().split(" | ").peekable();
        let mut prev_command:Option<Child>  = None;
        let mut read_from_map = false;
        let mut alias_key = String::new();
        while let Some(command) = if read_from_map {aliases.get(&alias_key).map(|s|s.as_str())} else {commands.next()} {
            let (command,args) = split_args(command.trim());
            read_from_map = false;
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
                    if commands.peek().is_none() {
                        print!("{}",ls_output);
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
                    let stdout = if commands.peek().is_some() {
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
                            read_from_map = true;
                            alias_key = command.to_string();
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


fn main() {
    // If you dont have gnome installed, remove everything in this main function and just leave the main_shell() functioncall
    if env::var("LAUNCHED_IN_TERMINAL").is_ok() {
        while main_shell(){println!("Restarted successfully")}
        return;
    }

    let current_exe = env::current_exe().unwrap();

    let result = Command::new("gnome-terminal")
        .arg("--")
        .arg(format!("{}",current_exe.to_str().unwrap()))
        .env("LAUNCHED_IN_TERMINAL", "1")
        .spawn();

        match result {
            Ok(_) => println!("Launched in a new terminal successfully!"),
            Err(e) => eprintln!("Failed to launch a new terminal: {}", e),
        }
}
