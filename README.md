# fdname - File and Directory Renaming Tool

fdname is a simple program that helps you batch rename files and directories.

## Modes of Operation

There are three commands available in fdname:
 - prefix
 - suffix
 - replace
 - remove
 - hash
 - lowercass
 - uppercase
 - whitespace

All are self-explanatory. Replace can also be used to "remove" by leaving the
second argument blank.

## Options

In addition to the modes of operation, there are a few options for fdname:
 - recursive renaming
 - files only
 - directories only.

By default, recursive renaming is off and renaming actions apply to both files
and directories. Renaming actions never apply to the "root" or starting directory.

## Example Usage

```
# Prefix all first level sub-directories of the current directory with _archived_
fdname -d prefix _archived_

# Suffix everything in the current directory with _bad_version
fdname -r suffix _bad_version

# Add a path to work in a specified directory
# Replace 2018_ with 2019_ for files only in a directory
fdname /path/to/directory -f replace 2018_ 2019_

# Remove __EDIT from files in the current directory
fdname -f remove __EDIT

# Hash everything in the current directory
fdname -r hash

# Options can be doubled up for more control
# Lowercase all files in the current directory
fdname -r -f lowercase

# Uppercase all sub-directories in the current directory
# Order of options does not matter
fdname -d -r uppercase

# Remove whitespace from everything in a directory
fdname /path/to/directory -r whitespace

# A little light reading
fdname --help
```