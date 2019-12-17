# Link Checker

Check for broken links in markdown files

## 🚧 Under Construction 🚧

Not ready yet. Still in development.

## Getting Started

To install a version of mlc go to the [gitlab releases page](https://gitlab.com/becheran/mlc/-/releases) and download the debian package.
You can also get the package for a specific version via the following web request. For example this will download the mlc package *version 0.1.1*:

``` bash
curl -L "https://gitlab.com/becheran/link-checker/-/jobs/artifacts/v0.1.1/download?job=build_debian_job" --output mlc.zip
```

Extract the content:

``` bash
unzip mlc.zip -d mlc
```

Install mlc via apt:

``` bash
apt install ./mlc/target/debian/mlc_0.1.1_amd64.deb

```

## Builds

You can download the latest linux ci build [here](https://gitlab.com/becheran/link-checker/-/jobs/artifacts/master/raw/target/release/mlc?job=build_linux_job).

## Release

To release a new version use the [cargo release](https://github.com/sunng87/cargo-release) repository. For example run:

``` bash
cargo release patch --skip-publish
```
