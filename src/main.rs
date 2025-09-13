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
    assume_yes: Option<bool>,
    install_assume_yes: Option<bool>,
    remove_assume_yes: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Symlink {
    link: String,
    target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Ubuntu {
    remove_snap_and_install_firefox_ppa: Option<bool>,
    remove_snap_and_install_firefox_ppa_yes_delete_my_bookmarks_and_everything: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Rustup {
    install_rust: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Localsetup {
    ssh: Option<SSH>,
    repositories: Option<Vec<Repository>>,
    packages: Option<Packages>,
    symlink: Option<Symlink>,
    ubuntu: Option<Ubuntu>,
    rustup: Option<Rustup>,
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

fn get_cache_path() -> String {
    if let Ok(path) = std::env::var("XDG_CACHE_HOME") {
        if path != "" {
            return path;
        }
    }
    if let Ok(path) = std::env::var("HOME") {
        if path != "" {
            return path + "/.cache";
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

fn get_setup(mut source: String) -> Result<Localsetup, Box<dyn std::error::Error>> {
    if source.starts_with("http://") || source.starts_with("https://") {
        if source.starts_with("https://github.com/") && source.contains("blob") {
            source = source.replacen("https://github.com/", "https://raw.githubusercontent.com/", 1).replacen("/blob/", "/refs/heads/", 1).to_owned();
        }
        let body = reqwest::blocking::get(source)?.text()?;
        let setup: Localsetup = toml::from_str(&body)?;
        return Ok(setup);
    } else {
        let mut file = std::fs::File::open(source)?;
        let mut config_toml = String::new();
        use std::io::Read;
        file.read_to_string(&mut config_toml)?;
        let setup: Localsetup = toml::from_str(&config_toml)?;
        return Ok(setup);
    }
}

fn path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

fn setup_ssh_key(no_passphrase: bool) -> Result<(), Box<dyn std::error::Error>> {
    print!("Setting up SSH key … ");

    if path_exists(&(get_home() + "/.ssh/id_ed25519.pub")) {
        println!("Already set up");
        return Ok(())
    }
    println!("");

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

    println!("=================== YOUR PUBLIC SSH KEY: ====================\n");
    println!("{}", public_key);
    println!("-------------------------------------------------------------");
    println!("Copy this key if you need to access to repositories via SSH.");
    println!("-------------------------------------------------------------");
    println!("\nPress enter to continue …");
    let _ = std::io::stdin().lines().next();
    Ok(())
}

fn setup_repository(repository: &Repository) -> Result<(), Box<dyn std::error::Error>> {
    let target = if let Some(target) = repository.target.strip_prefix("~/") {
        get_home() + "/" + target
    } else {
        repository.target.to_owned()
    };

    if path_exists(&target) {
        println!("Existing repository: {} -> {}", repository.source, repository.target);
        // Update?
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

    Ok(())
}

fn install_packages(packages: Vec<String>, assume_yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    print!("Installing packages … ");
    if packages.len() > 0 {
        println!("");
        let output = if assume_yes {
            std::process::Command::new("sudo")
                .arg("apt")
                .arg("install")
                .arg("--yes")
                .args(packages)
                .output()
                .expect("failed to execute process")
        } else {
            std::process::Command::new("sudo")
                .arg("apt")
                .arg("install")
                .args(packages)
                .output()
                .expect("failed to execute process")
        };

        println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
        println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
    } else {
        println!("No packages to install");
    }

    Ok(())
}

fn remove_packages(packages: Vec<String>, assume_yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    print!("Removing packages … ");
    if packages.len() > 0 {
        println!("");
        let output = if assume_yes {
            std::process::Command::new("sudo")
                .arg("apt")
                .arg("remove")
                .arg("--yes")
                .args(packages)
                .output()?
        } else {
            std::process::Command::new("sudo")
                .arg("apt")
                .arg("remove")
                .args(packages)
                .output()?
        };

        println!("{}", str::from_utf8(&output.stdout).unwrap());
        println!("{}", str::from_utf8(&output.stderr).unwrap());
    } else {
        println!("No packages to remove");
    }

    Ok(())
}

fn package_installed(package: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let status = std::process::Command::new("dpkg")
        .arg("-s")
        .arg(package)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    // Not, if a package is no longer installed, but the config files are still present, we will recognize this as
    // installed (for dpkg its: "Status: deinstall ok config-files"). For now, we're okay with that!
    return Ok(status.success())
}

const NOSNAPD_FILENAME: &str = "/etc/apt/preferences.d/nosnap.pref";
const NOSNAPD_FILE_CONTENT: &str = "
# To prevent repository packages from triggering the installation of snap,
# this file forbids snapd from being installed by APT.

Package: snapd
Pin: release a=*
Pin-Priority: -10
";

const FIREFOX_NOSNAP_FILENAME: &str = "/etc/apt/preferences.d/firefox-nosnap.pref";
const FIREFOX_NOSNAP_FILE_CONTENT: &str = "
Package: firefox*
Pin: release o=Ubuntu*
Pin-Priority: -1
";

const THUNDERBIRD_NOSNAP_FILENAME: &str = "/etc/apt/preferences.d/thunderbird-nosnap.pref";
const THUNDERBIRD_NOSNAP_FILE_CONTENT: &str = "
Package: thunderbird*
Pin: release o=Ubuntu*
Pin-Priority: -1
";

const FIREFOX_PPA_FILENAME: &str = "/etc/apt/preferences.d/firefox-ppa.pref";
const FIREFOX_PPA_FILE_CONTENT: &str = "
Package: firefox*
Pin: release o=LP-PPA-mozillateam
Pin-Priority: 501
";

const THUNDERBIRD_PPA_FILENAME: &str = "/etc/apt/preferences.d/thunderbird-ppa.pref";
const THUNDERBIRD_PPA_FILE_CONTENT: &str = "
Package: thunderbird*
Pin: release o=LP-PPA-mozillateam
Pin-Priority: 501
";

fn create_file_with_content_if_not_exists_root(filename: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if path_exists(filename) {
        return Ok(());
    }

    // TODO: Create dir if not exists

    let cache_path = get_cache_path();
    let mut file = std::fs::File::create(cache_path.clone() + "/localsetup/" + filename)?;
    use std::io::Write;
    file.write_all(content.as_bytes())?;

    std::process::Command::new("sudo")
                .arg("cp")
                .arg(cache_path + "/localsetup/" + filename)
                .arg(filename)
                .output()?;

    Ok(())
}

fn ubuntu_remove_snap_and_install_firefox_ppa(assume_yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    print!("Removing snap … ");
    if package_installed("snapd")? {
        print!("");

        if !assume_yes {
            println!("Removing snap and switching from snap-installed firefox and thundebird to PPA\nfirefox and thundebird will remove all bookmarks, setting, emails, and\neverything else. Are you sure you want that? [y/n]");
            loop {
                let input_line = std::io::stdin().lines().next().unwrap()?;
                if input_line == "n" {
                    return Ok(())
                }
                if input_line == "y" {
                    break;
                }
                println!("Please answer 'y' for yes or 'n' for no. Should all bookmarks, setting, emails,\nand everything else be deleted? [y/n]");
            }
        }

        let output = std::process::Command::new("sudo")
                .arg("apt")
                .arg("purge")
                .arg("--yes")
                .arg("snapd")
                .output()?;

        println!("{}", str::from_utf8(&output.stdout).unwrap());
        println!("{}", str::from_utf8(&output.stderr).unwrap());

        create_file_with_content_if_not_exists_root(NOSNAPD_FILENAME, NOSNAPD_FILE_CONTENT)?;
        create_file_with_content_if_not_exists_root(FIREFOX_NOSNAP_FILENAME, FIREFOX_NOSNAP_FILE_CONTENT)?;
        create_file_with_content_if_not_exists_root(THUNDERBIRD_NOSNAP_FILENAME, THUNDERBIRD_NOSNAP_FILE_CONTENT)?;
    } else {
        println!("snap already removed");
    }

    print!("Installing Firefox … ");
    if !package_installed("firefox")? {
        print!("");
        let output = std::process::Command::new("sudo")
            .arg("add-apt-repository")
            .arg("ppa:mozillateam/ppa")
            .output()?;

        println!("{}", str::from_utf8(&output.stdout).unwrap());
        println!("{}", str::from_utf8(&output.stderr).unwrap());

        let output = std::process::Command::new("sudo")
            .arg("apt")
            .arg("install")
            .arg("--yes")
            .arg("firefox")
            .arg("thunderbird")
            .output()?;

        println!("{}", str::from_utf8(&output.stdout).unwrap());
        println!("{}", str::from_utf8(&output.stderr).unwrap());

        create_file_with_content_if_not_exists_root(FIREFOX_PPA_FILENAME, FIREFOX_PPA_FILE_CONTENT)?;
        create_file_with_content_if_not_exists_root(THUNDERBIRD_PPA_FILENAME, THUNDERBIRD_PPA_FILE_CONTENT)?;
    } else {
        println!("Firefox is already installed");
    }

    Ok(())
}

fn ubuntu_specifics(ubuntu: &Ubuntu) -> Result<(), Box<dyn std::error::Error>> {
    if ubuntu.remove_snap_and_install_firefox_ppa.unwrap_or_default() {
        ubuntu_remove_snap_and_install_firefox_ppa(ubuntu.remove_snap_and_install_firefox_ppa_yes_delete_my_bookmarks_and_everything.unwrap_or_default())?;
    }

    Ok(())
}

fn file_to_list(filename: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::io::BufRead;

    let file = std::fs::File::open(filename)?;
    let lines = std::io::BufReader::new(file).lines();

    for line in lines {
        let line = line?;
        println!("{}", line);
    }

    Ok(Vec::new())
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

    println!("Using config file at: {}", config.source.as_ref().unwrap());

    let setup = get_setup(config.source.unwrap())?;

    #[cfg(debug_assertions)]
    println!("{:#?}", setup);

    if setup.ssh.as_ref().map(|ssh| ssh.setup_ssh_key.unwrap_or(true)).unwrap_or(true) {
        setup_ssh_key(setup.ssh.as_ref().map(|ssh| ssh.no_passphrase.unwrap_or(false)).unwrap_or(false))?;
    }

    if let Some(repositories) = setup.repositories.as_ref() {
        println!("Setting up repositories …");

        for repository in repositories {
            setup_repository(repository)?;
        }
    }

    if let Some(packages) = setup.packages.as_ref() {
        let mut packages_to_install = Vec::<String>::new();
        let mut packages_to_remove = Vec::<String>::new();

        if let Some(install) = &packages.install {
            packages_to_install.append(&mut install.clone());
        }
        if let Some(install_list) = &packages.install_list {
            packages_to_install.append(&mut file_to_list(&install_list)?);
        }

        if let Some(remove) = &packages.remove {
            packages_to_remove.append(&mut remove.clone());
        }
        if let Some(remove_list) = &packages.remove_list {
            packages_to_remove.append(&mut file_to_list(&remove_list)?);
        }

        let install_assume_yes = packages.install_assume_yes.unwrap_or(packages.assume_yes.unwrap_or(false));
        let remove_assume_yes = packages.remove_assume_yes.unwrap_or(packages.assume_yes.unwrap_or(false));

        install_packages(packages_to_install, install_assume_yes)?;
        remove_packages(packages_to_remove, remove_assume_yes)?;
    }

    if let Some(ubuntu) = setup.ubuntu.as_ref() {
        ubuntu_specifics(ubuntu)?;
    }


    Ok(())
}
