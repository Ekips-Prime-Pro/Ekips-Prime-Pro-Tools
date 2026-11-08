# Installation and releases

*Deutsche Version: [../de/install.md](../de/install.md)*

## Download (recommended)

Ready-made programs are on the
[releases page](https://github.com/Ekips-Prime-Pro/Ekips-Prime-Pro-Tools/releases).
Pick the file for your computer:

| Computer | File |
|---|---|
| Windows (most PCs) | `ekips-tools-<version>-x86_64-pc-windows-msvc.zip` |
| Windows on ARM | `ekips-tools-<version>-aarch64-pc-windows-msvc.zip` |
| Mac with Apple chip (M1, M2, …) | `ekips-tools-<version>-aarch64-apple-darwin.tar.gz` |
| Mac with Intel chip | `ekips-tools-<version>-x86_64-apple-darwin.tar.gz` |
| Linux (most PCs) | `ekips-tools-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Raspberry Pi (64-bit Raspberry Pi OS) | `ekips-tools-<version>-aarch64-unknown-linux-musl.tar.gz` |

Each archive contains `ekips` (command-line tool) and `ekips-lsp`
(language server). Unpack it and put the folder on your `PATH`, or call
the programs with their full path. `SHA256SUMS.txt` lets you check that the
download is complete.

On macOS, the first start may be blocked because the programs are not
signed. Allow them under *System Settings → Privacy & Security*, or run
`xattr -d com.apple.quarantine ekips ekips-lsp`.

The Linux programs are statically linked and run on every distribution.

## Building from source

You need [Rust](https://rustup.rs) (stable).

```sh
git clone https://github.com/Ekips-Prime-Pro/Ekips-Prime-Pro-Tools
cd Ekips-Prime-Pro-Tools
cargo install --path crates/cli    # installs `ekips`
cargo install --path crates/lsp    # installs `ekips-lsp`
```

## Making a release (maintainers)

1. Update `version` in the root `Cargo.toml` (`[workspace.package]`) and
   commit.
2. Create and push a tag that starts with `v`:
   ```sh
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. The workflow `.github/workflows/release.yml` builds all six targets,
   packages them with `README.md` and `LICENSE`, writes `SHA256SUMS.txt` and
   publishes a GitHub release with generated release notes. Tags with a `-`
   (e.g. `v0.2.0-beta.1`) become pre-releases.

A release for an existing tag can be rebuilt via *Actions → Release → Run
workflow*.

### Core dependency

`ekips-compiler` comes from the Core repository as a Git dependency in the
root `Cargo.toml`. It is currently pinned to a commit (`rev = …`). As soon
as the Core publishes tags (`v0.2.0`, …), switch to `tag = "v0.2.0"` and run
`cargo update -p ekips-compiler`.
