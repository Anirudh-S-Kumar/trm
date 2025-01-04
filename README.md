# `trm` - Temporary rm

`trm` is a temporary `rm` command that moves files to a temporary directory instead of deleting them. This is useful when you want to delete files but are not sure if you might need them later. Note that this program is not a replacement for `rm` and should not be used as such. A conscious decision has been taken to not follow the XDG Trash specification.


## Usage

```
trm - Temporary rm, a utility to reversibly remove your files

Usage: trm [OPTIONS] [FILES]... [COMMAND]

Commands:
  history  Shows history of all operations performed. For details on format for `before`, use --help
  purge    Purge from trash and also corresponding logs
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

## History of logs
```
Shows history of all operations performed. For details on format for `before`, use --help

Usage: trm history [OPTIONS]

Options:
  -a, --all              Show all the history
  -b, --before <BEFORE>  Show all changes before current time - given time
      --path <PATH>      Directory to see history of. If no path specified, will show history in cwd [default: ]
  -h, --help             Print help (see more with '--help')
```


By default, it will show all history of the current working directory:
```
$ trm history
```

But duration can be specified:
```
$ trm history --before 1d
```

Or for a specific directory:
```
$ trm history --path /path/to/dir
```

## Purge
```
Purge from trash and also corresponding logs

Usage: trm purge [OPTIONS]

Options:
  -b, --before <BEFORE>  Remove items before current time - given time. Follows same semantics as in history
  -h, --help             Print help
```

By default, it will remove files that are older than 30 days:
```
$ trm purge
```

But duration can be specified, which follow same semantics as in history:
```
$ trm purge --before 1d
```



## Notes

- If you do have `$XDG_DATA_HOME` set, the default directory will be `$XDG_DATA_HOME/trm_files`. Otherwise, it will be `/var/tmp/trm_files`.
- There is no way to recover files once they are purged, so be careful with this command.