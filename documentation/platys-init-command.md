# platys init

```
Initializes the current directory to be the root for a platys platform by creating an initial
config file, if one does not already exists The stack to use as well as its version need to be passed by the --stack and --stack-version options.
By default 'config.yml' is used for the name of the config file, which is created by the init

Usage:
  platys init [flags]

Flags:
  -c, --config-file string       The name of the local config file (defaults to config.yml) (default "config.yml")
  -y, --enable-services string   Comma separated list of services to enable in the config file
  -f, --force                    If specified, this command will overwrite any existing config file
  -h, --help                     help for init
  -x, --hw-arch string           Hardware architecture for the platform (default "x86-64")
  -n, --platform-name string     the name of the platform to generate.
  -e, --seed-config string       The name of a predefined stack to base this new platform on
  -s, --stack string             stack version to employ (default "trivadis/platys-modern-data-platform")
  -w, --stack-version string     version of the stack to employ (default "latest")
  -b, --structure string         defines the structure of the generated platform (flat = platform is generate on the level of the config.yml or subfolder = platform is generated into a subfolder)

Global Flags:
  -v, --verbose   verbose output (default true)
```

