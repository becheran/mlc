## Link checking

After you create or edit a markdown or html file, check its links with [mlc](https://github.com/becheran/mlc) before you report the task as done:

``` bash
mlc --offline --quiet --files "<comma separated list of the files you changed>"
```

* Exit code `0` means all links are fine. Exit code `1` means broken links. Each one is printed as `[Err ] <file> (<line>, <column>) => <link> - <reason>`.
* Fix every reported link. Correct the relative path or find the real URL. Do not guess a URL and do not remove a link just to make the check pass.
* Leave out `--offline` to also check web links. This needs network access and is slower, so use it when you added or changed external URLs.
* A link which is intentionally broken (example, placeholder) can be marked with `<!-- mlc-disable-next-line -->` in the line above.
* Anchors like `file.md#heading` are not validated by mlc. Check heading names yourself.
