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
        print!("({})[{}] > ",user.name().to_str().unwrap().green(),dir.blue());
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        
        let mut parts = input.trim().split_whitespace();
        let command = parts.next().unwrap();
        let args = parts;
        
        match command {

            "cd" => {
                let mut home : String = "/home/".to_owned();
                home.push_str(user.name().to_str().unwrap());
                dir = args.peekable().peek().map_or(home , |x| x.to_string());
                let path = Path::new(&dir);
                
                if let Err(e) = env::set_current_dir(&path){
                    eprintln!("{}",e);
                }
            },
            "exit" => return,
            command => {
            
                let mut child = Command::new(command)
                .args(args)
                .spawn()
                .unwrap();
            
                child.wait().unwrap();
            }
        }   
    }
}
