use std::fs;
use std::io::Read;
use users::{get_user_by_uid, get_current_uid};

pub fn get_home_usr() -> String{
    "/home/".to_owned() + get_user_by_uid(get_current_uid()).unwrap().name().to_str().unwrap()
}

pub fn read_commons(home_usr : String) -> Vec<String>{
    let common_file_path = home_usr + "/brick_shell/brick_shell_commons.txt"; // File to save common commands for autocomplete
    if !fs::exists(&common_file_path).unwrap(){
        fs::File::create(&common_file_path).unwrap(); // create if not exist
    }
    let mut commons = String::new();
    fs::File::open(common_file_path).unwrap().read_to_string(&mut commons).unwrap();

    let commons_vec: Vec<String> = commons.lines().map(String::from).collect();

    commons_vec


}

pub fn write_commons(home_usr : String,commons : Vec<String>){
    let common_file_path = home_usr + "/brick_shell/brick_shell_commons.txt"; 
    fs::write(&common_file_path, commons.join("\n")).unwrap();
}