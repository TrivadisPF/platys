# Overview of platys CLI

This page provides the usage information for the `platys` Command.

## Command options overview and help

You can also see this information by running `platys --help` from the command line.

```
Platys - Trivadis Platform in a Box - v 2.4.0
https://github.com/trivadispf/platys
Copyright (c) 2018-2020, Trivadis AG

Usage:
  platys [flags]
  platys [command]

Available Commands:
  clean         Cleans the contents in the $PATH/container-volume folder
  gen           Generates all the needed artifacts for the docker-based modern (data) platform
  help          Help about any command
  init          Initializes the current directory to be the root for the Modern (Data) Platform by creating an initial config file, if one does not already exists
  list_services List the services
  stacks        Lists the predefined stacks available for the init command
  version       Print the version number of platys

Flags:
  -h, --help      help for platys
  -v, --verbose   verbose output (default true)

Use "platys [command] --help" for more information about a command.
```
   
You can use platys binary, `platys [OPTIONS] [COMMAND] [ARGS...]`, to generate and manage docker compose files. 

### Use `--version` to show the version of `platys`

```
$ platys version
Platys - Trivadis Platform in a Box - v 2.4.0
https://github.com/trivadispf/platys
Copyright (c) 2018-2020, Trivadis AG
```
   
## Where to go next

* [Command line reference](../documentation/command-line-ref.md)
