use reqwest;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    source: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Ssh {
    setup_ssh_key: Option<bool>,
    no_passphrase: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Git {
    name: Option<String>,
    email: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Repository {
    source: String,
    target: String,
    update: Option<bool>,
    synchronise: Option<bool>,
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
    update_rust: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Uv {
    install_astral_sh: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Setupfile {
    packages: Option<Packages>,
    ssh: Option<Ssh>,
    git: Option<Git>,
    repositories: Option<Vec<Repository>>,
    symlink: Option<Symlink>,
    ubuntu: Option<Ubuntu>,
    rustup: Option<Rustup>,
    uv: Option<Uv>,
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

fn get_setup(mut source: String) -> Result<Setupfile, Box<dyn std::error::Error>> {
    if source.starts_with("http://") || source.starts_with("https://") {
        if source.starts_with("https://github.com/") && source.contains("blob") {
            #[cfg(debug_assertions)]
            print!("Redirecting {} to ", source);
            source = source.replacen("https://github.com/", "https://raw.githubusercontent.com/", 1).replacen("/blob/", "/refs/heads/", 1).to_owned();
            #[cfg(debug_assertions)]
            println!("{}", source);
        }
        let body = reqwest::blocking::get(source)?.text()?;
        let setup: Setupfile = toml::from_str(&body)?;
        return Ok(setup);
    } else {
        let mut file = std::fs::File::open(source)?;
        let mut config_toml = String::new();
        use std::io::Read;
        file.read_to_string(&mut config_toml)?;
        let setup: Setupfile = toml::from_str(&config_toml)?;
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

    println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
    println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));

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

fn setup_git(git: &Git) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(git_name) = git.name.as_ref() {
        if !std::process::Command::new("git")
            .arg("config")
            .arg("get")
            .arg("user.name")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()?
            .success() {
                let output = std::process::Command::new("git")
                    .arg("config")
                    .arg("--global")
                    .arg("user.name")
                    .arg(git_name)
                    .output()?;

                println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
                println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
            }
    }

    if let Some(git_email) = git.email.as_ref() {
        if !std::process::Command::new("git")
            .arg("config")
            .arg("get")
            .arg("user.email")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()?
            .success() {
                let output = std::process::Command::new("git")
                    .arg("config")
                    .arg("--global")
                    .arg("user.email")
                    .arg(git_email)
                    .output()?;

                println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
                println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
            }
    }

    Ok(())
}

fn setup_repository(repository: &Repository) -> Result<bool, Box<dyn std::error::Error>> {
    let target = if let Some(target) = repository.target.strip_prefix("~/") {
        get_home() + "/" + target
    } else {
        repository.target.to_owned()
    };

    if path_exists(&target) {
        println!("Existing repository: {} -> {}", repository.source, repository.target);
        return Ok(false)
    }

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

    Ok(true)
}

fn update_repository(repository: &Repository) -> Result<(), Box<dyn std::error::Error>> {
    let target = if let Some(target) = repository.target.strip_prefix("~/") {
        get_home() + "/" + target
    } else {
        repository.target.to_owned()
    };

    if !path_exists(&target) {
        println!("Error! Repository: {} -> {} does not exist!", repository.source, repository.target);
        // TODO: Fail!
    }

    println!("Updating repository: {} -> {}", repository.source, repository.target);

    let output = std::process::Command::new("git")
        .arg("pull")
        .current_dir(target)
        .output()
        .expect("failed to execute process");

    println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
    println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));

    Ok(())
}

fn synchronise_repository(repository: &Repository) -> Result<(), Box<dyn std::error::Error>> {
    let target = if let Some(target) = repository.target.strip_prefix("~/") {
        get_home() + "/" + target
    } else {
        repository.target.to_owned()
    };

    if !path_exists(&target) {
        println!("Error! Repository: {} -> {} does not exist!", repository.source, repository.target);
        // TODO: Fail!
    }

    println!("Synchronising repository: {} -> {}", repository.source, repository.target);

    let output = std::process::Command::new("git")
        .arg("commit")
        .arg("-am")
        .arg("autocommit")
        .current_dir(&target)
        .output()
        .expect("failed to execute process");

    println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
    println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));

    let output = std::process::Command::new("git")
        .arg("pull")
        .arg("-r")
        .current_dir(&target)
        .output()
        .expect("failed to execute process");

    println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
    println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));

    let output = std::process::Command::new("git")
        .arg("push")
        .current_dir(&target)
        .output()
        .expect("failed to execute process");

    println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
    println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));

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

        println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
        println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
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

    // Note, if a package is no longer installed, but the config files are still present, we will recognize this as
    // installed (for dpkg its: "Status: deinstall ok config-files"). For now, we're okay with that!
    //
    // Also, if a package is marked as "hold" we will recognize this as installed. TODO: fix that!
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

    let (directory, basename) = filename.rsplit_once("/").unwrap();
    let cache_path = get_cache_path();

    std::fs::create_dir_all(cache_path.clone() + "/localsetup/")?;
    let mut file = std::fs::File::create(cache_path.clone() + "/localsetup/" + basename)?;

    use std::io::Write;
    file.write_all(content.as_bytes())?;

    std::process::Command::new("mkdir")
                .arg("-p")
                .arg(directory)
                .output()?;

    std::process::Command::new("sudo")
                .arg("cp")
                .arg(cache_path + "/localsetup/" + basename)
                .arg(filename)
                .output()?;

    Ok(())
}

fn ubuntu_remove_snap_and_install_firefox_ppa(assume_yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    print!("Removing snap … ");
    if package_installed("snapd")? {
        println!("");

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

        println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
        println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
    } else {
        println!("snap already removed");
    }

    print!("Ensuring snap will never be installed again … ");
    if path_exists(NOSNAPD_FILENAME) && path_exists(FIREFOX_NOSNAP_FILENAME) && path_exists(THUNDERBIRD_NOSNAP_FILENAME) {
        println!("already safe");
    } else {
        create_file_with_content_if_not_exists_root(NOSNAPD_FILENAME, NOSNAPD_FILE_CONTENT)?;
        create_file_with_content_if_not_exists_root(FIREFOX_NOSNAP_FILENAME, FIREFOX_NOSNAP_FILE_CONTENT)?;
        create_file_with_content_if_not_exists_root(THUNDERBIRD_NOSNAP_FILENAME, THUNDERBIRD_NOSNAP_FILE_CONTENT)?;
        println!("done");
    }

    print!("Installing Firefox from PPA … ");
    if !package_installed("firefox")? {
        println!("");
        let output = std::process::Command::new("sudo")
            .arg("add-apt-repository")
            .arg("ppa:mozillateam/ppa")
            .output()?;

        println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
        println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));

        let output = std::process::Command::new("sudo")
            .arg("apt")
            .arg("install")
            .arg("--yes")
            .arg("firefox")
            .arg("thunderbird")
            .output()?;

        println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
        println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));

        create_file_with_content_if_not_exists_root(FIREFOX_PPA_FILENAME, FIREFOX_PPA_FILE_CONTENT)?;
        create_file_with_content_if_not_exists_root(THUNDERBIRD_PPA_FILENAME, THUNDERBIRD_PPA_FILE_CONTENT)?;
    } else {
        println!("Firefox is already installed");
    }

    print!("Ensuring Firefox will be installed from PPA … ");
    if path_exists(FIREFOX_PPA_FILENAME) && path_exists(THUNDERBIRD_PPA_FILENAME) {
        println!("already ensured");
    } else {
        create_file_with_content_if_not_exists_root(FIREFOX_PPA_FILENAME, FIREFOX_PPA_FILE_CONTENT)?;
        create_file_with_content_if_not_exists_root(THUNDERBIRD_PPA_FILENAME, THUNDERBIRD_PPA_FILE_CONTENT)?;
        println!("done");
    }

    Ok(())
}

fn ubuntu_specifics(ubuntu: &Ubuntu) -> Result<(), Box<dyn std::error::Error>> {
    if ubuntu.remove_snap_and_install_firefox_ppa.unwrap_or_default() {
        ubuntu_remove_snap_and_install_firefox_ppa(ubuntu.remove_snap_and_install_firefox_ppa_yes_delete_my_bookmarks_and_everything.unwrap_or_default())?;
    }

    Ok(())
}

fn setup_rustup(rustup: &Rustup) -> Result<(), Box<dyn std::error::Error>> {
    let rustup_installed = std::process::Command::new("which")
        .arg("rustup")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?
        .success();

    let rust_installed = std::process::Command::new("which")
        .arg("rustup")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?
        .success();

    if !rust_installed && rustup.install_rust.unwrap_or_default() {
        if !rustup_installed {
            println!("Installing rustup … ");

            let body = reqwest::blocking::get("https://sh.rustup.rs")?.text()?;

            let cache_path = get_cache_path();

            let mut file = std::fs::File::create(cache_path.clone() + "/rustup.sh")?;
            use std::io::Write;
            file.write_all(body.as_bytes())?;

            println!("Installing rust … ");

            let output = std::process::Command::new("sh")
                .arg(cache_path + "/rustup.sh")
                .arg("-y")
                .output()?;

            println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
            println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
        } else {
            println!("Rustup already installed");
            println!("Installing rust …");

            let output = std::process::Command::new("rustup")
                .arg("toolchain")
                .arg("install")
                .arg("stable")
                .output()?;

            println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
            println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
        }

    }

    if rustup_installed && rust_installed && rustup.update_rust.unwrap_or_default() {
        println!("Rustup already installed");
        println!("Rust already installed");
        println!("Updating rust …");

        let output = std::process::Command::new("rustup")
            .arg("update")
            .output()?;

        println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
        println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
    }

    Ok(())
}

fn setup_uv(uv: &Uv) -> Result<(), Box<dyn std::error::Error>> {
    let uv_installed = std::process::Command::new("which")
    .arg("uv")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()?
    .success();

    if !uv_installed && uv.install_astral_sh.unwrap_or_default() {
        println!("Downloading uv installer … ");

        let body = reqwest::blocking::get("https://astral.sh/uv/install.sh")?.text()?;

        let cache_path = get_cache_path();

        let mut file = std::fs::File::create(cache_path.clone() + "/uv_install.sh")?;
        use std::io::Write;
        file.write_all(body.as_bytes())?;

        println!("Installing uv … ");

        let output = std::process::Command::new("sh")
        .arg(cache_path + "/uv_install.sh")
        .output()?;

        println!(" | {}", str::from_utf8(&output.stdout).unwrap().replace("\n", "\n | "));
        println!(" | {}", str::from_utf8(&output.stderr).unwrap().replace("\n", "\n | "));
    } else {
        println!("uv already installed");
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

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn check_for_updates() -> Result<bool, Box<dyn std::error::Error>> {
    println!("Checking for updates …");

    #[derive(Deserialize)]
    struct GithubRelease {
        name: String,
    }
    let client = reqwest::blocking::Client::new();
    let latest_release = client.get("https://api.github.com/repos/zgtm/localsetup/releases/latest")
        .header(reqwest::header::USER_AGENT, "zgtm/localsetup 0.0.2")
        .send()?
        .json::<GithubRelease>().map_err(|e| {
            // Request again, so we can print the response
            let latest_release = client.get("https://api.github.com/repos/zgtm/localsetup/releases/latest")
                .header(reqwest::header::USER_AGENT, "zgtm/localsetup 0.0.2")
                .send()
                .map(|r| r.text().unwrap_or_default())
                .unwrap_or_default();
            println!("Latest release request response: {}", latest_release);
            e
        })?;

    let latest_version = latest_release.name.strip_prefix("Release v").unwrap_or(&latest_release.name);
    println!("Latest release: {}", latest_version);
    println!("Current version: {}", VERSION);

    if VERSION.split(".").map(|n| n.parse::<i32>().unwrap()).lt(latest_version.split(".").map(|n| n.parse::<i32>().unwrap())) {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn update() -> Result<(), Box<dyn std::error::Error>> {
    if !check_for_updates()? {
        return Ok(())
    }

    println!("Running update …");
    let _ = std::fs::remove_file(get_home() + "/.local/bin/__localsetup_old");
    let _ = std::fs::rename(get_home() + "/.local/bin/localsetup", get_home() + "/.local/bin/__localsetup_old");
    print!("Downloading update … ");
    let mut response = reqwest::blocking::get("https://github.com/zgtm/localsetup/releases/latest/download/localsetup")?;
    let mut file = std::fs::File::create(get_home() + "/.local/bin/localsetup")?;
    std::io::copy(&mut response, &mut file)?;
    println!("done");

    #[cfg(debug_assertions)]
    print!("Make file executable … ");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(get_home() + "/.local/bin/localsetup", std::fs::Permissions::from_mode(0o755))?;
    #[cfg(debug_assertions)]
    println!("done");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("localsetup version {}", VERSION);

    let mut args = std::env::args();
    let _ = args.next(); // Ignore program name
    let param = args.next();
    if args.next().is_some() {
        eprintln!("Error: Too many arguments.");
        return Ok(())
    }

    if let Some(param) = param {
        if param == "update" {
            return update();
        }

        let mut config = read_config()?;
        config.source = Some(param);

        // TODO: Maybe only write config if we cound read the setupfile
        write_config(config)?;
    }

    let config = read_config()?;

    println!("Using config file at: {}", config.source.as_ref().unwrap());

    let setup = get_setup(config.source.unwrap())?;

    #[cfg(debug_assertions)]
    println!("{:#?}", setup);

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

    if setup.ssh.as_ref().map(|ssh| ssh.setup_ssh_key.unwrap_or(true)).unwrap_or(true) {
        setup_ssh_key(setup.ssh.as_ref().map(|ssh| ssh.no_passphrase.unwrap_or(false)).unwrap_or(false))?;
    }

    if let Some(git) = setup.git.as_ref() {
        setup_git(&git)?;
    }

    if let Some(repositories) = setup.repositories.as_ref() {
        println!("Setting up repositories …");

        for repository in repositories {
            let newly_setup = setup_repository(repository)?;

            if !newly_setup {
                if repository.synchronise.unwrap_or_default() {
                    synchronise_repository(repository)?;
                } else if repository.update.unwrap_or_default() {
                    update_repository(repository)?;
                }
            }
        }
    }

    if let Some(ubuntu) = setup.ubuntu.as_ref() {
        ubuntu_specifics(ubuntu)?;
    }

    if let Some(rustup) = setup.rustup.as_ref() {
        setup_rustup(rustup)?;
    }

    if let Some(uv) = setup.uv.as_ref() {
        setup_uv(uv)?;
    }


    Ok(())
}
