# platys ui

```
Initializes the current directory to be the root for a platys platform by creating an initial
config file, if one does not already exists The stack to use as well as its version need to be passed by the --stack and --stack-version options.
By default 'config.yml' is used for the name of the config file, which is created by the init

Usage: platys ui [OPTIONS]

Options:
  -p, --port <PORT>                    port to bind to (0 = random available port) [default: 0]
  -v, --verbose                        Verbose output
      --no-browser                     Don't open the browser automatically but print the url by default
  -s, --stack <STACK>                  Stack image to pull services from [default: trivadis/platys-modern-data-platform]
  -w, --stack-version <STACK_VERSION>  Version of the stack [default: latest]
  -c, --config-file <CONFIG_FILE>      Config file to write when the user clicks Generate [default: config.yml]
  -h, --help                           Print help
```

