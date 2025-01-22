use std::env;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use users::{get_user_by_uid, get_current_uid};
use colored::Colorize;


fn main() {
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

            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;
        
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

                    let output = Command::new(command)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn();
                
                match output {
                    Ok(output) => {prev_command = Some(output);},
                    Err(e) => {prev_command = None; eprintln!("{}",e);}
                    }
                }
            }
        }
        if let Some(mut final_cmd) = prev_command {
            final_cmd.wait().unwrap();
        }   
    }
}
