# Markup Link Checker

[![crates.io](https://img.shields.io/crates/v/mlc.svg?color=orange)](https://crates.io/crates/mlc)
[![downloads](https://badgen.net/crates/d/mlc?color=blue)](https://crates.io/crates/mlc)
[![build status](https://github.com/becheran/mlc/actions/workflows/rust.yml/badge.svg)](https://github.com/becheran/mlc/actions/workflows/rust.yml)
[![license](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/license/mit)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/becheran/mlc/blob/master/CONTRIBUTING.md)

![image](./docs/mlc.gif)

Check for broken links in markup files. Currently `html` and `markdown` files are supported. The Markup Link Checker can easily be integrated in your CI pipeline to prevent broken links in your markup docs.

## Features

* Find and check links in `markdown` and `html` files
* Validated absolute and relative file paths and URLs
* User friendly command line interface
* Easy [CI pipeline integration](#ci-pipeline)
* Very fast execution using [async rust](https://rust-lang.github.io/async-book/)
* Efficient link resolving strategy which tries with minimized network load
* Throttle option to prevent *429 Too Many Requests* errors
* Report broken links via GitHub workflow commands

## Install Locally

There are different ways to install and use *mlc*.

### Cargo

Use rust's package manager [cargo](https://doc.rust-lang.org/cargo/) to install *mlc* from [crates.io](https://crates.io/crates/mlc):

``` bash
cargo install mlc
```

### Download Binaries

To download a compiled binary version of *mlc* go to [github releases](https://github.com/becheran/mlc/releases) and download the binaries compiled for:
- **Linux**: x86_64 and aarch64 (arm64)
- **macOS**: aarch64 (Apple Silicon)
- **Windows**: x86_64

### Arch Linux

You can install from the [official repositories](https://archlinux.org/packages/extra/x86_64/markuplinkchecker/) using [pacman](https://wiki.archlinux.org/title/Pacman):

```bash
pacman -S markuplinkchecker
```

## CI Pipeline

### GitHub Actions

Use *mlc* in GitHub using the *GitHub-Action* from the [Marketplace](https://github.com/marketplace/actions/markup-link-checker-mlc).

``` yaml
- name: Markup Link Checker (mlc)
  uses: becheran/mlc@v1
```

Use *mlc* command line arguments using the `with` argument:

``` yaml
- name: Markup Link Checker (mlc)
  uses: becheran/mlc@v1
  with:
    args: ./README.md
```

The action does uses [GitHub workflow commands](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands) to highlight broken links:

![annotation](./docs/FailingAnnotation.PNG)

### Binary

To integrate *mlc* in your CI pipeline running in a *linux x86_64 environment* you can add the following commands to download and execute it:

``` bash
curl -L https://github.com/becheran/mlc/releases/download/v1.0.0/mlc-x86_64-linux -o mlc
chmod +x mlc
./mlc
```

For **linux aarch64/arm64** environments, use:

``` bash
curl -L https://github.com/becheran/mlc/releases/download/v1.0.0/mlc-aarch64-linux -o mlc
chmod +x mlc
./mlc
```

For example take a look at the [ntest repo](https://github.com/becheran/ntest/blob/master/.github/workflows/ci.yml) which uses *mlc* in the CI pipeline.

## Docker

Use the *mlc* docker image from the [docker hub](https://hub.docker.com/r/becheran/mlc) which includes *mlc*:

``` sh
docker run becheran/mlc mlc
```

## Usage

Once you have *mlc* installed, it can be called from the command line. The following call will check all links in markup files found in the current folder and all subdirectories:

``` bash
mlc
```

Another example is to call *mlc* on a certain directory or file:

``` bash
mlc ./docs
```

Alternatively you may want to ignore all files currently ignored by `git` (requires `git` binary to be found on $PATH) and set a root-dir for relative links:

```bash
mlc --gitignore --root-dir .
```

Call *mlc* with the `--help` flag to display all available cli arguments:

``` bash
mlc -h
```

The following arguments are available:

| Argument         | Short | Description |
|------------------|-------|-------------|
| `<directory>`    |       | Only positional argument. Path to directory which shall be checked with all sub-dirs. Can also be a specific filename which shall be checked. |
| `--help`         | `-h`  | Print help |
| `--debug`        | `-d`  | Show verbose debug information |
| `--do-not-warn-for-redirect-to` | | Do not warn for links which redirect to the given URL. Allows the same link format as `--ignore-links`. For example, `--do-not-warn-for-redirect-to "http*://crates.io*"` will not warn for links which redirect to the `crates.io` website. |
| `--offline`      | `-o`  | Do not check any web links. Renamed from `--no-web-links` which is still an alias for downwards compatibility |
| `--match-file-extension` | `-e`  | Set the flag, if the file extension shall be checked as well. For example the following markup link `[link](dir/file)` matches if for example a file called `file.md` exists in `dir`, but would fail when the `--match-file-extension` flag is set. |
| `--version`      | `-V` | Print current version of mlc |
| `--ignore-path`  | `-p` | Comma separated list of directories or files which shall be ignored. For example  |
| `--gitignore`    | `-g` | Ignore all files currently ignored by git (requires `git` binary to be available on $PATH). |
| `--gituntracked` | `-u` | Ignore all files currently untracked by git (requires `git` binary to be available on $PATH). |
| `--ignore-links` | `-i` | Comma separated list of links which shall be ignored. Use simple `?` and `*` wildcards. For example `--ignore-links "http*://crates.io*"` will skip all links to the crates.io website. See the [used lib](https://github.com/becheran/wildmatch) for more information.  |
| `--markup-types` | `-t` | Comma separated list list of markup types which shall be checked. Possible values: `md`, `html` |
| `--root-dir`     | `-r` | All links to the file system starting with a slash on linux or backslash on windows will use another virtual root dir. For example the link in a file `[link](/dir/other/file.md)` checked with the cli arg `--root-dir /env/another/dir` will let *mlc* check the existence of `/env/another/dir/dir/other/file.md`. |
| `--throttle`     | `-T` | Number of milliseconds to wait in between web requests to the same host. Default is zero which means no throttling. Set this if you need to slow down the web request frequency to avoid `429 - Too Many Requests` responses. For example with `--throttle 15`, between each http check to the same host, 15 ms will be waited. Note that this setting can slow down the link checker. |
| `--csv`          |      | Path to csv file which contains all failed requests and warnings in the format `source,line,column,target,severity`. The severity column contains `ERR` for errors and `WARN` for warnings. |

All optional arguments which can be passed via the command line can also be configured via the `.mlc.toml` config file in the working directory:

``` toml
# Print debug information to console
debug = true
# Do not warn for links which redirect to the given URL
do-not-warn-for-redirect-to=["http*://crates.io*"]
# Do not check web links
offline = true
# Check the exact file extension when searching for a file
match-file-extension= true
# List of files and directories which will be ignored
ignore-path=["./ignore-me","./src"]
# Ignore all files ignored by git
gitignore = true
# List of links which will be ignored
ignore-links=["http://ignore-me.de/*","http://*.ignoresub-domain/*"]
# List of markup types which shall be checked
markup-types=["Markdown","Html"]
# Wait time in milliseconds between http request to the same host
throttle= 100
# Path to the root folder used to resolve all relative paths
root-dir="./"
# Path to csv file which contains all failed requests and warnings
csv="output.csv"
```

## Changelog

Checkout the [changelog file](https://github.com/becheran/mlc/blob/master/CHANGELOG.md) to see the changes between different versions.

## License

This project is licensed under the *MIT License* - see the [LICENSE file](https://github.com/becheran/mlc/blob/master/LICENSE) for more details.
