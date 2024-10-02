# eph

Manage and run *"ephemeral"* scripts you don't want cluttering your workspace.

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
