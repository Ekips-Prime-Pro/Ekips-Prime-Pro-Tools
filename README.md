# Ekips Prime Pro – Tools

Command-line tool, language server and simulator for
[Ekips Prime Pro](https://github.com/Ekips-Prime-Pro/ekips-prime-pro-core),
the TypeScript-like robot language for the LEGO® SPIKE™ Prime hub.

```text
$ ekips check mission.ekips
error: I don't understand the word `cosnt` here.
 --> mission.ekips:1:1
  |
1 | cosnt arm = new Motor(Port.C);
  | ^^^^^
  = help: Did you mean `const`?

mission.ekips: 1 error
```

All messages are available in English and German (`--lang en|de`, default:
system language).

## Parts

| Part | Crate | Status |
|---|---|---|
| `ekips` CLI | [`crates/cli`](crates/cli) | `check` works; `run --sim`, `upload`, `monitor`, `flash-firmware`, `backup-firmware` are announced but not available yet |
| Language server `ekips-lsp` | [`crates/lsp`](crates/lsp) | Diagnostics while typing, hover, autocompletion with snippets |
| Simulator | [`crates/sim`](crates/sim) | Skeleton + design; waits for the VM in Core |

The compiler comes from the Core repository (`ekips-compiler`, Git
dependency pinned to a commit until Core publishes tags).

## Documentation

See [`docs/`](docs/README.md) – every page in English and German:
[CLI](docs/en/cli.md) · [language server](docs/en/lsp.md) ·
[simulator design](docs/en/simulator.md) · [installation](docs/en/install.md).

## Development

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p ekips-cli -- check path/to/program.ekips
```

CI runs on Linux (x86_64 and aarch64), Windows and macOS. Pushing a `v*`
tag builds release binaries for all platforms. Contributor rules are in
[`CLAUDE.md`](CLAUDE.md).

## License

[MIT](LICENSE)
