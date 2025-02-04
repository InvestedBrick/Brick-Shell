use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use serde_json;

pub fn write_aliases(filename : String,map: &HashMap<String,String>){
    let json = serde_json::to_string(map).unwrap();
    let mut file = File::create(filename).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}

pub fn read_aliases(filename : String) -> HashMap<String,String> {
    let mut file = match File::open(filename){
        Ok(f) => f,
        Err(_) => {return HashMap::<String,String>::new()}
    };
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let map: HashMap<String,String> = serde_json::from_str(&contents).unwrap();
    map
}