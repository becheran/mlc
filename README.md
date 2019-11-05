# Link Checker

Check for broken links in markdown files

## 🚧 Under Construction 🚧

Not ready yet. Still in development.

## Getting Started

To install a version of the linkchecker go to the [gitlab releases page](./-/releases) and download the debian package.
You can also get the package for a specific version via the following web request. For example this will download the linkchecker package *version 0.1.1*:
```
curl -L "https://gitlab.com/becheran/link-checker/-/jobs/artifacts/v0.1.1/download?job=release_debian_job" --output linkchecker.zip
```

## Builds

You can download the latest linux ci build [here](https://gitlab.com/becheran/link-checker/-/jobs/artifacts/master/raw/target/release/linkchecker?job=build_linux_job).

## Release

To release a new version use the [cargo release](https://github.com/sunng87/cargo-release) repository. For example run:
```
cargo release patch --skip-publish
```