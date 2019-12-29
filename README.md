# Markup Link Checker

[![](http://meritbadge.herokuapp.com/mlc)](https://crates.io/crates/mlc)
[![](https://badgen.net/crates/d/mlc)](https://crates.io/crates/mlc)
[![Build Status](https://gitlab.com/becheran/mlc_ci/badges/master/pipeline.svg)](https://gitlab.com/becheran/mlc_ci/pipelines)
[![](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Check for broken links in markdown files. Can easily be integrated in your CI/CD pipeline to prevent broken links in your docs.

## Install

There are different ways to install and use mlc.

### Cargo

Use rust's package manager [cargo](https://doc.rust-lang.org/cargo/) to install mlc from [crates.io](https://crates.io/crates/mlc):

``` bash
cargo install mlc
```

### Download Binaries

To download a compiled binary version of mlc go to [github releases](https://github.com/becheran/mlc/releases) and download the binaries compiled for `x86_64-unknown-linux-gnu`.

### CI/CD Pipeline Integration

To integrate mlc in your CI pipeline running in a linux x86_64 environment you can add the following commands to download mlc:

``` bash
curl -L https://github.com/becheran/mlc/releases/download/0.6.3/mlc -o mlc
chmod +x mlc
```

For example take a look at the [ntest repo](https://github.com/becheran/ntest) which uses mlc in the CI pipeline.

## Usage

Once you have mlc installed it canned just be called from the command line. The following call will check all links in markup files found in the current folder and all subdirectories:

``` bash
mlc
```

Another example would be to call mlc on a certain directory or file:

``` bash
mlc ./docs
```

Call mlc with the `--help` flag to display all available cli arguments:

``` bash
mlc -h
```

## Changelog

Checkout the [changelog file](https://github.com/becheran/mlc/blob/master/CHANGELOG.md) to see the changes between different versions.

## Contribution

All contributions and comments welcome! Open an issue or create a Pull Request whenever you find a bug or have an idea to improve this crate.

## License

This project is licensed under the MIT License - see the [LICENSE file](https://github.com/becheran/mlc/blob/master/LICENSE) for details.

## Planned Features

- Timeout for requests as cl argument
- Improve speed
- Add .ignore file support
- Support other markup files such as tex or html
