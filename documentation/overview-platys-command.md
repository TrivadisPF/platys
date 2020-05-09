# Overview of platys CLI

This page provides the usage information for the `platys` Command.

## Command options overview and help

You can also see this information by running `platys --help` from the command line.

```
Usage: platys [OPTIONS] COMMAND [ARGS]...

Options:
  --version   Show the version and exit.
  -h, --help  Show this message and exit.

Commands:
  clean
  config         Sets configuration
  gen            Generates all the needed artefacts for the docker-based...
  init           Initialises the current directory to be the root for the...
  list_services  Shows the services interfaces of the stack, web and/or apis
  stacks         Lists the predefined stacks available for the init
  start          Starts the stack, once it is generated.
  stop           Stops the stack, once it is generated.
  upload-stack   Uploads the stack to a remote machine
```
   
You can use platys binary, `platys [OPTIONS] [COMMAND] [ARGS...]`, to generate and manage docker compose files. 

### Use `--version` to show the version of `platys`

```
$ platys --version
Trivadis Docker-based Modern Data Platform Generator v2.2.0   
```
   
## Where to go next

* [Command line reference](../documentation/command-line-ref.md)
