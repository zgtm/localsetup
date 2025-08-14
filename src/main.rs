use reqwest;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    source: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Localsetup {
    setup_ssh_key: Option<String>, 
}

fn get_config_path() -> String {
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        if path != "" {
            return path;
        }
    }
    if let Ok(path) = std::env::var("HOME") {
        if path != "" {
            return path + "/.config";
        }
    }
    return "".to_string();
}

fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    let file = std::fs::File::open(config_path + "/localsetup.toml");
    if let Ok(mut file) = file {
        let mut config_toml = String::new();
        use std::io::Read;
        file.read_to_string(&mut config_toml)?;
        let config: Config = toml::from_str(&config_toml)?;
        return Ok(config);
    }
    
    Ok(Config{source: None})
}

fn write_config(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_toml = toml::to_string(&config)?;
    let config_path = get_config_path();
    let mut file = std::fs::File::create(config_path + "/localsetup.toml")?;
    use std::io::Write;
    file.write_all(config_toml.as_bytes())?;
    Ok(())
}

fn get_setup(source: String) -> Result<Localsetup, Box<dyn std::error::Error>> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let body = reqwest::blocking::get(source)?.text()?;
        let setup: Localsetup = toml::from_str(&body)?;
        return Ok(setup);
    }

    let mut file = std::fs::File::open(source)?;
    let mut config_toml = String::new();
    use std::io::Read;
    file.read_to_string(&mut config_toml)?;
    let setup: Localsetup = toml::from_str(&config_toml)?;
    return Ok(setup);
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let _ = args.next(); // Ignore program name
    let param = args.next();
    if args.next().is_some() {
        eprintln!("Error: Too many arguments.");
        return Ok(())
    } 
    
    if let Some(config_source) = param {
        let mut config = read_config()?;
        config.source = Some(config_source);
        write_config(config)?;
    }

    let config = read_config()?;
    let setup = get_setup(config.source.unwrap())?;
    
    println!("setup = {setup:?}");

    Ok(())
}
