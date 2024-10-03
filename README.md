# eph

Manage and run *"ephemeral"* scripts you don't want cluttering your workspace.

For when you:
* don't want to check a shell script in
* don't want to edit the gitignore
* don't want to rely solely on terminal history
* want to edit multi-line shell commands in an editor
* & dont want to clean up scripts before committing

```
Usage: eph [OPTIONS] [script] [args]...

Arguments:
  [script]   Script to run
  [args]...  Arguments for the script

Options:
  -l, --list       List all scripts
  -e <SCRIPT>      Edit an existing script
  -n <SCRIPT>      Create a new script
  -d <SCRIPT>      Delete a script
  -h, --help       Print help
```

### Config

```
# ~/.config/eph/config.toml
editor = <editor>
script_dir = <dir> # Optional, defaults to ~/.eph/
```
