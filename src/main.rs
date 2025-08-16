use reqwest;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    source: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct SSH {
    setup_ssh_key: Option<bool>,
    no_passphrase: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Repository {
    source: String,
    target: String,
    update: Option<bool>,
    synchronize: Option<bool>,
    run_once: Option<String>,
    run_everytime: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Packages {
    install: Option<Vec<String>>,
    remove: Option<Vec<String>>,
    install_list: Option<String>,
    remove_list: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Symlink {
    link: String,
    target: String,
}


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Localsetup {
    ssh: Option<SSH>,
    repositories: Option<Vec<Repository>>,
    packages: Option<Packages>,
    symlink: Option<Symlink>,
}

fn get_home() -> String {
    return std::env::var("HOME").unwrap_or_default()
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

fn path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

fn setup_ssh_key(no_passphrase: bool) -> Result<(), Box<dyn std::error::Error>> {
    if path_exists(&(get_home() + "/.ssh/id_ed25519.pub")) {
        return Ok(())
    }

    let output = if no_passphrase {
        std::process::Command::new("ssh-keygen")
            .arg("-t")
            .arg("ed25519")
            .arg("-f")
            .arg(&(get_home() + "/.ssh/id_ed25519"))
            .arg("-N")
            .arg("")
            .output()
            .expect("failed to execute process")
    } else {
        std::process::Command::new("ssh-keygen")
            .arg("-t")
            .arg("ed25519")
            .arg("-f")
            .arg(&(get_home() + "/.ssh/id_ed25519"))
            .output()
            .expect("failed to execute process")
    };

    println!("{}", str::from_utf8(&output.stdout).unwrap());
    println!("{}", str::from_utf8(&output.stderr).unwrap());

    let mut file = std::fs::File::open(&(get_home() + "/.ssh/id_ed25519.pub"))?;
    let mut public_key = String::new();
    use std::io::Read;
    file.read_to_string(&mut public_key)?;

    println!("\n========== YOUR PUBLIC SSH KEY ===========");
    print!("{}", public_key);
    println!("==========================================");

    println!("\nCopy this key if you need to access to repositories via SSH.\nPress enter to continue …");
    let _ = std::io::stdin().lines().next();
    Ok(())
}

fn setup_repositories(repositories: &[Repository]) -> Result<(), Box<dyn std::error::Error>> {
    for repository in repositories {
        let target = if let Some(target) = repository.target.strip_prefix("~/") {
            get_home() + "/" + target
        } else {
            repository.target.to_owned()
        };

        if path_exists(&target) {
            println!("Existing repository: {} -> {}", repository.source, repository.target);
            // Upadet?
        } else {
            println!("Creating repository: {} -> {}", repository.source, repository.target);
            let (mut base, dir) = target.rsplit_once("/").expect("Invalid directory");
            if dir == "" {
                (base, _)  = base.rsplit_once("/").expect("Invalid directory");
            }

            if !path_exists(base) {
                std::fs::create_dir_all(base)?;
            }

            std::process::Command::new("git")
                .arg("clone")
                .arg(&repository.source)
                .arg(&target)
                .output()
                .expect("failed to execute process");

        }
    }
    Ok(())
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

    if setup.ssh.as_ref().map(|ssh| ssh.setup_ssh_key.unwrap_or(true)).unwrap_or(true) {
        setup_ssh_key(setup.ssh.as_ref().map(|ssh| ssh.setup_ssh_key.unwrap_or(false)).unwrap_or(false))?;
    }

    if let Some(repositories) = setup.repositories.as_ref() {
        setup_repositories(repositories)?;
    }


    Ok(())
}
