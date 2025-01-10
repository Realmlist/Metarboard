use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    interval: i32,
    station: String,
    // TODO: expand later
}

pub fn read_config_file() {
    // Load in cross-platform directories in order to gather config file
    // Examples:
    // Linux:   /home/alice/.config/metarboard
    // Windows: C:\Users\Alice\AppData\Roaming\Realmlist\Metarboard
    // macOS:   /Users/Alice/Library/Application Support/eu.Realmlist.Metarboard
    if let Some(proj_dirs) = ProjectDirs::from("eu", "Realmlist", "Metarboard") {
        let config_dir = proj_dirs.config_dir();
        let config_file =
            fs::read_to_string(config_dir.join("metarboard.toml")).unwrap_or("".to_string());
        // Create path & default config if it doesn't exist
        if !config_dir.exists() {
            match fs::create_dir_all(config_dir) {
                Ok(_) => println!("Config directory created successfully!"),
                Err(e) => println!("Error creating directory: {}", e),
            }
        }
        let config_file_path = config_dir.join("metarboard.toml");
        if !config_file_path.exists() {
            create_config(config_file_path);
        }

        //Load into struct
        let config: Config = toml::from_str(&config_file).unwrap();

        // TODO: parse config file & use vars
        dbg!(config);
    }
}

fn create_config(config_file_location: PathBuf) {
    println!("Config file doesn't exist! Creating new one...");
    let config = Config {
        interval: 30,
        station: "EHGR".to_string(),
    };

    // Serialize
    let toml = match toml::to_string(&config) {
        Ok(toml) => toml,
        Err(e) => {
            eprintln!("Error serializing config: {}", e);
            return;
        }
    };

    // Open the file (this will create a new file or overwrite an existing one)
    let mut file = match File::create(&config_file_location) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error creating file: {}", e);
            return;
        }
    };

    // Write the TOML string to the file
    if let Err(e) = file.write_all(toml.as_bytes()) {
        eprintln!("Error writing to file: {}", e);
    } else {
        println!("TOML configuration written to metarboard.toml");
    }
}
