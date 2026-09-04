# Overview of platys CLI

This page provides the usage information for the `platys` Command.

## Command options overview and help

You can also see this information by running `platys --help` from the command line.

```
Platys - Trivadis Platform in a Box - v 3.1.1
https://github.com/trivadispf/platys
Copyright (c) 2018-2026, Trivadis AG

Usage: platys [OPTIONS] <COMMAND>

Commands:
  version        Print the version number of platys
  init           Initializes the current directory to be the root for the Modern (Data) Platform by creating an initial config file, if one does not already exist
  gen            Generates all the needed artifacts for the docker-based modern (data) platform
  clean          Cleans the contents in the $PATH/container-volume folder
  list-services  List the services contained in the given version of the platys tool
  stacks         Lists the predefined stacks available for the init command
  ui
  help           Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose
          Verbose output

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
   
You can use platys binary, `platys [OPTIONS] [COMMAND] [ARGS...]`, to generate and manage docker compose files. 

### Use `--version` to show the version of `platys`

```
$ platys version
Platys - Trivadis Platform in a Box - v 3.1.1
https://github.com/trivadispf/platys
Copyright (c) 2018-2026, Trivadis AG
```
   
## Where to go next

* [Command line reference](../documentation/command-line-ref.md)
