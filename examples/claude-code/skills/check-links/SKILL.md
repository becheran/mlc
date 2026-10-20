---
name: check-links
description: Check and fix broken links in markdown and html files with mlc. Use after creating or editing documentation, READMEs, changelogs or any other markup file, and when the user asks to verify, check or fix links.
---

# Check links with mlc

[mlc](https://github.com/becheran/mlc) finds broken links in markdown and html files. It must be on the `PATH` (`cargo install mlc`, or a binary from the [releases](https://github.com/becheran/mlc/releases)).

## Steps

1. Check the files you changed. Without network access this is fast:

   ``` bash
   mlc --offline --quiet --files "README.md,docs/guide.md"
   ```

   To check a whole directory use `mlc --offline --quiet docs/`. To check all markup files changed on the branch:

   ``` bash
   mlc --offline --quiet --files "$(git diff --name-only --diff-filter=d origin/main... -- '*.md' '*.html' | paste -sd,)"
   ```

2. If you added or changed external URLs, also check them (needs network, slower):

   ``` bash
   mlc --quiet --throttle 100 --files "README.md"
   ```

3. Exit code `0` means all links are fine. Exit code `1` means broken links. Each one is printed to stderr as `[Err ] <file> (<line>, <column>) => <link> - <reason>`.

4. Fix every reported link:
   - Local file not found: search for the file and correct the relative path.
   - URL not found: search for the real page or ask the user. Never invent a replacement URL.
   - Intentionally broken link (example, placeholder): add `<!-- mlc-disable-next-line -->` in the line above.
   - Never delete a link only to make the check pass.

5. Run the check again until the exit code is `0`.

## Limits

- Anchors like `file.md#heading` are not validated, only the file or URL part. Check heading names yourself.
- Links in code blocks and inline code are ignored.
- Some sites block bots. If a URL works in the browser but fails here, add `--http-headers "User-Agent: Mozilla/5.0"`. For `429 - Too Many Requests` increase `--throttle`.
- For programmatic use `mlc --format json` prints one JSON document: `{"summary": {...}, "issues": [{"severity", "source", "line", "column", "target", "message"}]}`.
