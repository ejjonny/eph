use clap::{Arg, ArgAction, Command};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{Result, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

fn main() -> Result<()> {
    let matches = Command::new("eph")
        .about("Manage and run ephemeral scripts")
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List all scripts")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("edit")
                .short('e')
                .num_args(1)
                .value_name("SCRIPT")
                .help("Edit an existing script"),
        )
        .arg(
            Arg::new("new")
                .short('n')
                .num_args(1)
                .value_name("SCRIPT")
                .help("Create a new script"),
        )
        .arg(
            Arg::new("delete")
                .short('d')
                .num_args(1)
                .value_name("SCRIPT")
                .help("Delete a script"),
        )
        .arg(Arg::new("script").help("Script to run").index(1))
        .arg(
            Arg::new("args")
                .help("Arguments for the script")
                .index(2)
                .action(ArgAction::Append),
        )
        .get_matches();

    let config_dir = dirs::home_dir()
        .map(|p| p.join(".config/eph"))
        .expect("Could not find home directory");
    fs::create_dir_all(&config_dir)?;

    let config_file_path = config_dir.join("config.toml");

    let config = load_or_create_config(&config_file_path)?;

    let script_dir = if let Some(dir) = &config.script_dir {
        PathBuf::from(dir)
    } else {
        dirs::home_dir()
            .map(|p| p.join(".eph"))
            .expect("Could not find home directory")
    };
    fs::create_dir_all(&script_dir)?;

    if matches.get_flag("list") {
        list_scripts(&script_dir)?;
    } else if let Some(script_name) = matches.get_one::<String>("edit") {
        edit_script(script_dir, script_name, &config)?;
    } else if let Some(script_name) = matches.get_one::<String>("new") {
        create_script(script_dir, script_name, &config)?;
    } else if let Some(script_name) = matches.get_one::<String>("delete") {
        delete_script(script_dir, script_name)?;
    } else if let Some(script_name) = matches.get_one::<String>("script") {
        let script_args: Vec<&String> = matches
            .get_many::<String>("args")
            .map_or(vec![], |vals| vals.collect());
        run_script(script_dir, script_name, &script_args)?;
    } else {
        eprintln!("No valid command provided. Use --help for usage.");
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Config {
    editor: Option<String>,
    script_dir: Option<String>,
}

fn load_or_create_config(config_file_path: &Path) -> Result<Config> {
    if config_file_path.exists() {
        let contents = fs::read_to_string(config_file_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    } else {
        let default_config = Config {
            editor: Some("nano".to_string()),
            script_dir: None,
        };
        let toml_string = toml::to_string_pretty(&default_config).unwrap();
        let mut file = fs::File::create(config_file_path)?;
        file.write_all(toml_string.as_bytes())?;
        Ok(default_config)
    }
}

fn edit_script(script_dir: PathBuf, script_name: &str, config: &Config) -> Result<()> {
    let script_path = script_dir.join(script_name);
    if !script_path.exists() {
        eprintln!("Script does not exist. Use -n to create a new script.");
        return Ok(());
    }
    open_in_editor(script_path, config)
}

fn create_script(script_dir: PathBuf, script_name: &str, config: &Config) -> Result<()> {
    let script_path = script_dir.join(script_name);
    if script_path.exists() {
        eprintln!("Script already exists. Use -e to edit.");
        return Ok(());
    }
    fs::write(&script_path, b"#!/bin/zsh\n\n")?;
    let mut perms = fs::metadata(&script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms)?;
    open_in_editor(script_path, config)
}

fn delete_script(script_dir: PathBuf, script_name: &str) -> Result<()> {
    let script_path = script_dir.join(script_name);
    if script_path.exists() {
        let trash_dir = script_dir.join("trash");
        fs::create_dir_all(&trash_dir)?;
        let trash_path = trash_dir.join(script_name);
        fs::rename(&script_path, &trash_path)?;
        println!("Script '{}' moved to trash.", script_name);
    } else {
        eprintln!("Script '{}' does not exist.", script_name);
    }
    Ok(())
}

fn run_script(script_dir: PathBuf, script_name: &str, args: &[&String]) -> Result<()> {
    let script_path = script_dir.join(script_name);
    if !script_path.exists() {
        eprintln!("Script '{}' does not exist.", script_name);
        return Ok(());
    }
    let current_dir = env::current_dir()?;
    let status = ProcessCommand::new(&script_path)
        .args(args)
        .current_dir(current_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        eprintln!("Script exited with status: {}", status);
    }
    Ok(())
}

fn open_in_editor(script_path: PathBuf, config: &Config) -> Result<()> {
    let editor = config.editor.clone().unwrap_or_else(|| "nano".to_string());
    let status = ProcessCommand::new(editor).arg(script_path).status()?;
    if !status.success() {
        eprintln!("Editor exited with status: {}", status);
    }
    Ok(())
}

fn list_scripts(script_dir: &PathBuf) -> Result<()> {
    let mut scripts = Vec::new();

    for entry in fs::read_dir(script_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if !name_str.starts_with('.') && name_str != "trash" {
                        scripts.push(name_str.to_string());
                    }
                }
            }
        }
    }

    if scripts.is_empty() {
        println!("No scripts found.");
    } else {
        println!("Available scripts:");
        for script in scripts {
            println!("- {}", script);
        }
    }

    Ok(())
}
