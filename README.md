# platys



## Project structure

```
src/
  main.rs            # entry point — parses CLI and dispatches
  cli.rs             # clap CLI definition 
  docker.rs          # Docker helpers (pull_config, get_file, stop_remove_container)
  config.rs          # YAML types, validators, node-limit map, regex helpers
  commands/
    mod.rs
    version.rs       # platys version
    init.rs          # platys init
    gen.rs           # platys gen
    clean.rs         # platys clean
    list_services.rs # platys list_services
    stacks.rs        # platys stacks
assets/
  init_banner.txt    # embedded at compile time via include_str!
```

## Building

```bash
cargo build --release
# binary at: target/release/platys
```

Requires Docker to be running 

## Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
bollard = "0.17"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
yaml_serde = "0.10"
reqwest = { version = "0.13", features = ["blocking"] }
anyhow = "1"

futures-util = "0.3"
tar = "0.4"
tempfile = "3"
log = "0.4.29"
libc ="0.2.186"
env_logger = "0.11.10"
indexmap = {version = "2", features = ["serde"]}

[dev-dependencies]
indoc = "2"

```

On Linux/macOS, add `libc = "0.2"` to `Cargo.toml` if you need the UID/GID
helpers in `gen.rs` (used to run the container as the current user).


