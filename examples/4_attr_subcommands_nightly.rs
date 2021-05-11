use clapi::macros::*;

/// #### Requires Nightly Rust #####

#[command(
    name="datamap",
    description="get or store data by a key-value pair",
    version="1.0"
)]
fn main() {}

#[subcommand(name="list", description="list the stored key-values")]
#[option(pretty, alias="p", description="pretty print the values")]
fn list_data(pretty: bool) {
    let map = utils::load();

    if pretty {
        let json = serde_json::to_string_pretty(&map);
        match json {
            Ok(s) => {
                s.lines().for_each(|s| {
                    println!("{}", s);
                })
            },
            Err(_) => {}
        }

    } else {
        for (key, value) in map {
            println!("{} = {}", key, value)
        }
    }
}

#[subcommand(name="get", description="get a value by it's key")]
#[arg(key)]
fn get_data(key: String) {
    let map = utils::load();

    match map.get(&key) {
        Some(s) => println!("{}", s),
        None => println!("No value found for '{}'", key)
    };
}

#[subcommand(name="set", description="sets a key-value pair")]
#[arg(key)]
#[arg(value)]
fn set_data(key: String, value: String) {
    if key.is_empty() {
        println!("Keys cannot be empty")
    } else {
        let mut map = utils::load();
        map.insert(key, value);
        utils::save(map).unwrap()
    }
}

mod utils {
    use std::fs::{OpenOptions, File};
    use std::collections::HashMap;
    use std::io::{Read, Write, Error};
    use std::path::PathBuf;

    static FILE_NAME : &str = "target/datamap.data";

    fn file_path() -> PathBuf {
        let mut dir = std::env::current_dir().unwrap();
        dir.push(FILE_NAME);
        dir
    }

    pub fn load_raw_json() -> Result<String, Error> {
        let mut buf = String::new();
        let path = file_path();
        let mut file: File;

        if path.exists() {
            file = OpenOptions::new()
                .read(true)
                .open(file_path())
                .unwrap()
        } else {
            file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(file_path())
                .unwrap()
        }

        match file.read_to_string(&mut buf) {
            Ok(_) => Ok(buf),
            Err(e) => Err(e)
        }
    }

    pub fn load() -> HashMap<String, String> {
        let json = load_raw_json();

        let result = match json {
            Ok(ref s) => serde_json::from_str(s),
            Err(_) => return HashMap::default(),
        };

        match result {
            Ok(data) => data,
            Err(_) => HashMap::default()
        }
    }

    pub fn save(data: HashMap<String, String>) -> Result<(), serde_json::Error>{
        let json = serde_json::to_string(&data);

        match json {
            Ok(s) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .open(file_path())
                    .unwrap();

                file.write_all(s.as_bytes()).unwrap();
                file.flush().unwrap();
                Ok(())
            },
            Err(e) => Err(e)
        }
    }
}
