use clap::{Arg, ArgAction, Command};
use std::env;
use std::fs;
use std::io::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};

fn main() -> Result<()> {
    let matches = Command::new("eph")
        .version("1.0")
        .about("Manage and run ephemeral scripts")
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

    let script_dir = dirs::home_dir()
        .map(|p| p.join(".eph"))
        .expect("Could not find home directory");
    fs::create_dir_all(&script_dir)?;

    if let Some(script_name) = matches.get_one::<String>("edit") {
        edit_script(script_dir, script_name)?;
    } else if let Some(script_name) = matches.get_one::<String>("new") {
        create_script(script_dir, script_name)?;
    } else if let Some(script_name) = matches.get_one::<String>("delete") {
        delete_script(script_dir, script_name)?;
    } else if let Some(script_name) = matches.get_one::<String>("script") {
        let script_args: Vec<&String> = matches
            .get_many::<String>("args")
            .map_or(vec![], |vals| vals.collect());
        run_script(script_dir, script_name, script_args.as_slice())?;
    } else {
        eprintln!("No valid command provided. Use --help for usage.");
    }

    Ok(())
}

fn edit_script(script_dir: PathBuf, script_name: &str) -> Result<()> {
    let script_path = script_dir.join(script_name);
    if !script_path.exists() {
        eprintln!("Script does not exist. Use -n to create a new script.");
        return Ok(());
    }
    open_in_editor(&script_path)
}

fn create_script(script_dir: PathBuf, script_name: &str) -> Result<()> {
    let script_path = script_dir.join(script_name);
    if script_path.exists() {
        eprintln!("Script already exists. Use -e to edit.");
        return Ok(());
    }
    fs::write(&script_path, b"#!/bin/zsh\n\n")?;
    let mut perms = fs::metadata(&script_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms)?;
    open_in_editor(&script_path)
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

fn open_in_editor(script_path: &PathBuf) -> Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "hx".to_string());
    let status = ProcessCommand::new(editor).arg(script_path).status()?;
    if !status.success() {
        eprintln!("Editor exited with status: {}", status);
    }
    Ok(())
}
