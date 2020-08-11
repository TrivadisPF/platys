# platys gen

```
Generates all the needed artifacts for the docker-based modern (data) platform.
    		The stack configuration can either be passed as a local file (using the --config-filename option or using the default name 'config.yml')
			or as an URL
    		referencing a file on the Internet (using the --config-url option).

Usage:
  platys gen [flags]

Flags:
  -c, --config-file string   The name of the local config file (defaults to config.yml) (default "config.yml")
  -u, --config-url string    The URL to a remote config file
  -l, --del-empty-lines      Remove empty lines from the docker-compose.yml file. (default true)
  -h, --help                 help for gen

Global Flags:
  -v, --verbose   verbose output (default true)
```
