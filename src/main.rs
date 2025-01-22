use std::vec::IntoIter;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use users::{get_user_by_uid, get_current_uid};
use colored::Colorize;
use std::fs;

fn split_args(command : &str) -> (&str, IntoIter<String>){
    if let Some((command_,args_)) = command.split_once(' '){
        let mut result_args: Vec<String> = Vec::new();
        let mut in_quotes = false;
        let mut current_arg = String::new();

        for char in args_.chars() {
            match char {
                '"' => {
                    in_quotes = !in_quotes;
                    current_arg.push(char);
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

fn main_shell() {
    let user = get_user_by_uid(get_current_uid()).unwrap();
    let mut dir : String = String::new();
    loop {
        dir = env::current_dir().unwrap().display().to_string();
        print!("({})[{}] > ",user.name().to_str().unwrap().green().bold(),dir.blue().bold());
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        
        let mut commands = input.trim().split(" | ").peekable();
        let mut prev_command  = None;

        while let Some(command) = commands.next() {
            let (command,args) = split_args(command.trim());
        
            match command {

                "cd" => {
                    let mut home : String = "/home/".to_owned();
                    home.push_str(user.name().to_str().unwrap());
                    let new_dir = args.peekable().peek().map_or(home , |x| x.to_string());

                    let path = Path::new(&new_dir);

                    if let Err(e) = env::set_current_dir(&path){
                        eprintln!("{}",e);
                    }

                    prev_command = None;
                },
                "exit" => return,
                "ls" => {
                    let files = fs::read_dir(&dir).unwrap();
                    for file in files {
                        let file_type = file.as_ref().unwrap().file_type().unwrap();
                        let file_name = file.unwrap().file_name().into_string().unwrap();
                        if file_type.is_dir() {
                            if file_name.starts_with("."){
                                print!("{}/ ",file_name.purple().bold());
                            }else {
                                print!("{}/ ",file_name.blue().bold());
                            }
                        }else if file_type.is_file() {
                            if !file_name.ends_with(".tmp"){

                                if file_name.contains("."){
                                    print!("{} ",file_name);
                                }else{
                                    print!("{} ",file_name.green().bold());
                                }
                            }
                        }else if file_type.is_symlink() {
                            print!(">{} ",file_name.bright_cyan())
                        }
                    }
                    println!("");
                },
                command => {
                    
                    let stdin = prev_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };
                    //for arg in args.clone(){
                    //    print!("{arg} ");
                    //}
                    //println!("")
                    let output = Command::new(command)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn();
                
                match output {
                    Ok(output) => {prev_command = Some(output);},
                    Err(_e) => {prev_command = None; eprintln!("Command '{}' was not found!",command.to_string());}
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
    if env::var("LAUNCHED_IN_TERMINAL").is_ok() {
        main_shell();
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
