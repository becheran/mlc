# Markup Link Checker Reference

## CLI Arguments

| Argument         | Short | Description |
|------------------|-------|-------------|
| `--help`         | `-h`  | Print help |
| `--debug`        | `-d`  | Show verbose debug information |
| `--no-web-links` |       | Do not check any web links |
| `--version`      | `-V`  | Print current version of mlc |
| `--ignore-links` |       | List of links which shall be ignored. Use simple `?` and `*` wildcards. For example `--ignore-links "http*://crates.io*"` will skip all links to the crates.io website. See the [used lib](https://github.com/becheran/wildmatch) for more information.  |
| `--markup-types` | `-t`  | List of markup types which shall be checked [possible values: md, html] |
| `<directory>`    |       | Path to directory which shall be checked with all sub-dirs. Can also be a specific filename which shall be checked. |
