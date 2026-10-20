# AGENTS.md

This file is for AI coding agents. The first part explains how to use *mlc* in a project. The second part is for working on *mlc* itself.

## Using mlc in a project

*mlc* (markup link checker) finds broken links in `markdown` and `html` files. Use it after you created or edited documentation, README files, changelogs or any other markup file, and before you tell the user that the work is done.

### Commands

``` bash
# Check the files you edited. No network access, fast and deterministic
mlc --offline --quiet --files "README.md,docs/guide.md"

# Check a directory or a single file
mlc --offline --quiet docs/

# Also check external URLs. Needs network access and takes longer
mlc --quiet --throttle 100 docs/

# Machine readable result
mlc --format json docs/
```

If *mlc* is not installed, see the [installation section](./README.md#install-locally). In short: `cargo install mlc`, or download a binary from the [releases page](https://github.com/becheran/mlc/releases).

### Reading the result

* Exit code `0`: no broken links. Exit code `1`: at least one broken link. Warnings do not change the exit code.
* Every broken link is one line on *stderr*: `[Err ] <file> (<line>, <column>) => <link> - <reason>`.
* With `--format json` *stdout* contains exactly one JSON document: `{"summary": {...}, "issues": [{"severity", "source", "line", "column", "target", "message"}]}`.

### Fixing broken links

1. Local file not found: look for the file with a search tool and correct the relative path. Never delete a link just to make the check pass.
2. External URL not found (`404`): do not guess a replacement. Search for the real page or ask the user.
3. If a link is intentionally broken (example, placeholder, private site), add `<!-- mlc-disable-next-line -->` in the line above or use `--ignore-links "<pattern>"`.
4. Run *mlc* again until the exit code is `0`.

### Limits

* Anchors (`file.md#heading`) are not validated, only the file or URL part of the link.
* Links in code blocks and inline code are ignored.
* `--gituntracked` skips untracked files, it does not select them. Use `--files` to select files.

Details: [README](./README.md#ai-agents-and-automation). Copy-paste integrations for `AGENTS.md`, Claude Code hooks and skills, and pre-commit are in [examples/](./examples/README.md).

## Working on mlc

``` bash
cargo build
cargo test            # unit and integration tests, the web link tests use a mock server
cargo fmt
cargo clippy --all-targets
cargo run -- ./README.md --offline    # check the links of the readme
```

* The crate uses Rust edition 2024 with the minimum supported Rust version in `rust-version` of [Cargo.toml](./Cargo.toml). The `msrv` CI job checks it, so do not use newer language or library features without raising it.
* Layout: [src/cli.rs](./src/cli.rs) parses arguments and `.mlc.toml` into the [config](./src/config.rs). [src/lib.rs](./src/lib.rs) finds the links, [src/checker.rs](./src/checker.rs) checks the distinct targets concurrently, [src/printer.rs](./src/printer.rs) prints while doing so and [src/report.rs](./src/report.rs) is the result which is rendered as summary, JSON or CSV.
* A new or changed command line argument needs a matching entry in the `.mlc.toml` config (`OptionalConfig` in [src/config.rs](./src/config.rs)), the argument table and config example in the [README](./README.md), and the `Unreleased` section of the [CHANGELOG](./CHANGELOG.md).
* `OptionalConfig` implements `Default`. Use `..Default::default()` when building it in new tests.
* Everything the program prints to *stdout* has to stay free of banner or log lines in `--format json` mode. [tests/cli_output.rs](./tests/cli_output.rs) runs the real binary to guard this.
