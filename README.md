# `trm` - Temporary rm

`trm` is a temporary `rm` command that moves files to a temporary directory instead of deleting them. This is useful when you want to delete files but are not sure if you might need them later. Note that this program is not a replacement for `rm` and should not be used as such. A conscious decision has been taken to not follow the XDG Trash specification.


## Usage

```
trm - Temporary rm, a utility to reversibly remove your files

Usage: trm [OPTIONS] [FILES]... [COMMAND]

Commands:
  history  Shows history of all operations performed. For details on format for `before`, use --help
  help     Print this message or the help of the given subcommand(s)

Arguments:
  [FILES]...  Files to delete

Options:
  -v, --verbose    Display full file paths or not
      --debug      Debug output
  -u, --undo       
  -l, --list       Display all files trashed under given directories. Takes current directory as default if no other directory given
  -d, --dir <DIR>  Directory where to move [default: /var/tmp/trm_files]
  -h, --help       Print help
  -V, --version    Print version
```

Basic usage:
```
$ trm file1 dir1/
```

Undo:
```
$ trm -u file1 dir1/
```

List all trashed files:
```
$ trm -l
```

To recover all files trashed in current directory:
```
$ trm -lu
```

## Notes

- If you do have `$XDG_DATA_HOME` set, the default directory will be `$XDG_DATA_HOME/trm_files`. Otherwise, it will be `/var/tmp/trm_files`.
