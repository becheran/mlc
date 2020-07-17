# Markup Link Checker Reference

## CLI Arguments

| Argument         | Short | Description |
|------------------|-------|-------------|
| `--help`         | `-h`  | Print help |
| `--debug`        | `-d`  | Show verbose debug information |
| `--no-web-links` |       | Do not check any web links |
| `--match-file-extension`|| Set the flag, if the file extension shall be checked as well. For example the following markup link `[link](dir/file)` matches if for example a file called `file.md` exists in `dir`, but would fail when the `--match-file-extension` flag is set. |
| `--version`      | `-V`  | Print current version of mlc |
| `--ignore-path`  |       | List of directories or files which shall be ignored. |
| `--ignore-links` |       | List of links which shall be ignored. Use simple `?` and `*` wildcards. For example `--ignore-links "http*://crates.io*"` will skip all links to the crates.io website. See the [used lib](https://github.com/becheran/wildmatch) for more information.  |
| `--markup-types` | `-t`  | List of markup types which shall be checked [possible values: md, html] |
| `--root-dir` | `-r`  | All links to the file system starting with a slash on linux or backslash on windows will use another virtual root dir. For example the link in a file `[link](/dir/other/file.md)` checked with the cli arg `--root-dir /env/another/dir` will let *mlc* check the existence of `/env/another/dir/dir/other/file.md`. |
| `<directory>`    |       | Path to directory which shall be checked with all sub-dirs. Can also be a specific filename which shall be checked. |
| `--throttle`     |       | Number of milliseconds to wait in between web requests. Default is zero which means no throttling. Set this if you need to slow down the web request frequency to avoid `429 - Too Many Requests` responses. For example with `--throttle 15`, between each http check 15 ms will be waited. Note that this setting can slow down the link checker significantly. |
