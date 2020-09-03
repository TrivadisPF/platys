# `platys` - What's new?

## What's new in 2.4.0

Platys 2.4.0 contains the following new functionalities and enhancements:

### Enhancements

* the output of "platys list_services" is now ordered

### Bug fixes

* if there is an error in the generator (inside docker-compose-templer), it is not shown on the Platys console
* "platys init" with the "-f" (force) flag does not work correctly
* Wrong values for global configuration after "platys init --enable-services" 
* After running "platys gen", the "generated" folders are owned by root

## What's new in 2.3.0

Platys 2.3.0 is a port from the previous Python based implementation to a Golang based version. The main reason for that was to get better portability between Mac, Linux and Windows. 

### Enhancements

* added the Platys Slack Workspace to the [Platys homepage](../README.md)
* the short arguments of the various commands (`gen`, `init`, ...) had to be renamed (as go only supports one letter short arguments names).
* the following platys properties in config.yml have been renamed (replace these if you have an existing `platys` platform):
  *  from `stack-image-name` to `platform-stack`
  *  from `stack-image-version` to `platform-stack-version`

## What's new in 2.2.0

Platys 2.2.0 contains the following new functionalities and enhancements:

### New Functionality

* first version of a `clean` command

### Enhancements

* `--structure` flag moved from `gen` to `init` command 

## What's new in 2.1.0

Platys 2.1.0 contains the following new functionalities and enhancements:

### New Functionality

* supports CentOS 7

## What's new in 2.0.0

Platys 2.0.0 contains the following new functionalities and enhancements:

### New Functionality

* supports windows


