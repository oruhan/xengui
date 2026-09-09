# xengui CLI

`xengui` is the XenGui workspace's development and release command-line tool. It discovers the workspace through Cargo metadata, so commands work from the repository root or any member directory.

Install it locally with:

```bash
cargo install --path crates/xengui-cli
```

Use `xengui --help` or a command's `--help` output for the complete interface. Version-changing commands preserve TOML comments and formatting through `toml_edit`; `version set` and `version bump` require `--write`, while `version suggest` requires `--apply` before modifying files.

The SemVer suggestion is intentionally conservative. Removed public Rust declarations, removed Cargo features, or an explicit `BREAKING CHANGE` marker recommend `major`; added public API or features recommend `minor`; fixes, documentation, tests, tooling, and internal-only changes recommend `patch`. The CLI always prints its reason before an optional apply.
