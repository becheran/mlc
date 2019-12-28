# Markup Link Checker

[![](http://meritbadge.herokuapp.com/mlc)](https://crates.io/crates/mlc)
[![Build Status](https://gitlab.com/becheran/mlc_ci/badges/master/pipeline.svg)](https://gitlab.com/becheran/mlc_ci/pipelines)

Check for broken links in markdown files. Can easily be integrated in your CI/CD pipeline to prevent broken links in your docs.

## Getting Started

To install a version of mlc go to [gitlab releases](https://gitlab.com/becheran/mlc/-/releases) and download the binaries.

Or use rust's package manager [cargo](https://doc.rust-lang.org/cargo/) to install mlc:

``` bash
cargo install mlc
```

Once you have mlc installed it canned just be called from the command line:

``` bash
mlc
```

Another example would be to call mlc on a certain dir/file:

``` bash
mlc ./docs
```

## Changelog

Checkout the [changelog file](https://gitlab.com/becheran/mlc/blob/master/CHANGELOG.md) to see the changes between different versions.

## Contribution

All contributions and comments welcome! Open an issue or create a Pull Request whenever you find a bug or have an idea to improve this crate.

## License

This project is licensed under the MIT License - see the [LICENSE file](https://gitlab.com/becheran/mlc/blob/master/LICENSE) for details.

## Planned Features

- Timeout for requests as cl argument
- Improve speed
- Add .ignore file support
- Support other markup files such as tex or html
- Add docker image with latest installed version for easy CI pipeline integration
