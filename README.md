# Markup Link Checker

Check for broken links in markdown files.

## 🚧 Under Construction 🚧

Not ready yet. Still in development.

Planned features till a stable release (v1.*):

- Bug fixes
- Check mail addresses
- Update docs
- Support markup reference links
- Timeout for requests as cl argument

Planned for the future:

- Improve speed
- Add .ignore file support
- Support other markup files such as tex or html
- Add docker image with latest installed version

## Getting Started

To install a version of mlc go to the [gitlab releases page](https://gitlab.com/becheran/mlc/-/releases) and download the binaries.

Or use rust's package manager [cargo](https://doc.rust-lang.org/cargo/) to install mlc:

``` bash
cargo install mlc
```

Once you have mlc installed it canned just be called from the commandline:

``` bash
mlc
```

Another example would be to call mlc on a certain dir/file:

``` bash
mlc ./docs
```

## Changelog

Checkout the [changelog file](./CHANGELOG.md) to see the changes between different versions.

## Contribution

All contributions and comments welcome! Open an issue or create a Pull Request whenever you find a bug or have an idea to improve this crate.

## License

This project is licensed under the MIT License - see the [LICENSE file](./LICENSE) for details.
