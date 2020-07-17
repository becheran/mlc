# Markup Link Checker

[![](http://meritbadge.herokuapp.com/mlc)](https://crates.io/crates/mlc)
[![](https://badgen.net/crates/d/mlc)](https://crates.io/crates/mlc)
[![Build Status](https://gitlab.com/becheran/mlc_ci/badges/master/pipeline.svg)](https://gitlab.com/becheran/mlc_ci/pipelines)
[![](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![asciicast](https://asciinema.org/a/299100.svg)](https://asciinema.org/a/299100)

Check for broken links in markup files. Currently `html` and `markdown` files are supported. The Markup Link Checker can easily be integrated in your CI/CD pipeline to prevent broken links in your markup docs.

Fast execution because of [async](https://rust-lang.github.io/async-book/) usage.

## Install

There are different ways to install and use *mlc*.

### Cargo

Use rust's package manager [cargo](https://doc.rust-lang.org/cargo/) to install *mlc* from [crates.io](https://crates.io/crates/mlc):

``` bash
cargo install mlc
```

### Download Binaries

To download a compiled binary version of *mlc* go to [github releases](https://github.com/becheran/mlc/releases) and download the binaries compiled for `x86_64-unknown-linux-gnu`.

### CI/CD Pipeline Integration

**Binary**

To integrate *mlc* in your CI pipeline running in a *linux x86_64 environment* you can add the following commands to download the tool:

``` bash
curl -L https://github.com/becheran/mlc/releases/download/v0.13.0/mlc -o mlc
chmod +x mlc
```

For example take a look at the [ntest repo](https://github.com/becheran/ntest/blob/master/.gitlab-ci.yml) which uses *mlc* in the CI pipeline.

**Docker**

Use the *mlc* docker image from the [docker hub](https://hub.docker.com/repository/docker/becheran/mlc) which includes *mlc*.

## Usage

Once you have *mlc* installed, it can be called from the command line. The following call will check all links in markup files found in the current folder and all subdirectories:

``` bash
mlc
```

Another example is to call *mlc* on a certain directory or file:

``` bash
mlc ./docs
```

Call *mlc* with the `--help` flag to display all available cli arguments:

``` bash
mlc -h
```

See the [reference](./docs/reference.md) for all available command line arguments.

## Changelog

Checkout the [changelog file](https://github.com/becheran/mlc/blob/master/CHANGELOG.md) to see the changes between different versions.

## License

This project is licensed under the *MIT License* - see the [LICENSE file](https://github.com/becheran/mlc/blob/master/LICENSE) for more details.
