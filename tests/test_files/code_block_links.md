# Test Code Block Links

This file tests the raw link checking feature in code blocks.

## Regular link (should always be checked)
[Regular link](http://example.com/)

## Code block with raw URLs (checked by default, can be disabled)

```bash
# Download config files
wget http://example.com/config.yml
curl https://example.org/data.json
```

## Inline code with URL
Use `curl http://example.com/api` to fetch data.

## Another code block
```
http://example.net/path
```
