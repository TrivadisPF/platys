# platys — Rust Port

This is a faithful translation of [TrivadisPF/platys](https://github.com/TrivadisPF/platys) from Go to idiomatic Rust.

## Go → Rust mapping

| Go concept | Rust equivalent |
|---|---|
| `cobra` CLI framework | `clap` with `derive` feature |
| `docker/docker` SDK | `bollard` async Docker SDK |
| `gopkg.in/yaml.v3` | `serde_yaml` |
| goroutines / channels | `tokio` async/await |
| `embed.FS` | `include_str!` macro |
| `panic(err)` | `anyhow::bail!` / `?` operator |
| `log.Fatal(...)` | `anyhow::bail!` |
| `io.Copy` stream | `futures_util::StreamExt` |
| `tar.NewReader` | `tar::Archive` |
| `http.Get` (blocking) | `reqwest::blocking::get` |
| `os/user.Current()` | `libc::getuid/getgid` (Unix) |

## Project structure

```
src/
  main.rs            # entry point — parses CLI and dispatches
  cli.rs             # clap CLI definition (replaces cmd/root.go)
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

Requires Docker to be running (same as the Go original).

## Dependencies

```toml
clap        = "4"          # CLI parsing (cobra replacement)
bollard     = "0.17"       # Docker API
tokio       = "1"          # async runtime
serde       = "1"          # serialisation
serde_yaml  = "0.9"        # YAML
reqwest     = "0.12"       # HTTP (for --config-url)
anyhow      = "1"          # ergonomic error handling
regex       = "1"          # service key matching
futures-util= "0.3"        # stream helpers
tar         = "0.4"        # tar extraction
tempfile    = "3"          # temp files for remote configs
```

On Linux/macOS, add `libc = "0.2"` to `Cargo.toml` if you need the UID/GID
helpers in `gen.rs` (used to run the container as the current user).

## Notable translation decisions

- **Error handling**: Go's `panic(err)` / `log.Fatal(err)` become `bail!` or `?`
  propagated up through `anyhow::Result`. All errors surface cleanly at `main`.
- **Async**: The Go code uses the Docker SDK synchronously in goroutines.
  The Rust port uses `bollard`'s native async API with a `tokio` runtime.
- **Embed**: Go's `//go:embed assets` becomes Rust's `include_str!` macro,
  which embeds `assets/init_banner.txt` at compile time.
- **YAML node walking**: Go's `gopkg.in/yaml.v3` exposes raw `*yaml.Node` trees
  for surgical editing. `serde_yaml` uses a `Value` enum which serves the same
  purpose in `init.rs`.
- **`stacks` command**: The Go version has a TODO comment about regex filtering
  the container logs. The Rust port implements that TODO using `regex`.
- **Windows support**: The `gen` command skips the `User` field on Windows,
  matching the Go `runtime.GOOS == "windows"` branch.
EOF