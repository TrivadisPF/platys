# platys gen

```
Usage: platys gen [OPTIONS]

  Generates all the needed artifacts for the docker-based modern (data)
  platform.

  The stack configuration can either be passed as a local file (using the
  --config-filename option or using the default name 'config.yml') or as an
  URL referencing a file on the Internet (using the --config-url option).

Options:
  -cf, --config-filename TEXT   the name of the local config file.  [default:
                                config.yml]
  -cu, --config-url TEXT        the URL to a remote config file
  -de, --del-empty-lines TEXT   remove empty lines from the docker-compose.yml
                                file.  [default: True]
  --structure [flat|subfolder]  defines the structure of the generated
                                platform (deprecated and now part of init) -
                                flat = platform is generate on the level of
                                the config.yml or - subfolder = platform is
                                generated into a subfolder, named to the value
                                of --platform-name
  -v, --verbose                 Verbose logging  [default: False]
  -h, --help                    Show this message and exit.
```
