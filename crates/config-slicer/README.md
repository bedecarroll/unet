# `config-slicer`

`config-slicer` parses hierarchical match expressions and applies them to
configuration text so focused slices and diffs can be generated from the
command line.

## Workflows

### Parse a match expression

```bash
config-slicer parse --match "system||ntp"
config-slicer parse --match "system||ntp" --json
```

### Slice a configuration

Reads from a file when one is provided, otherwise reads from stdin.

```bash
config-slicer slice --match "system||ntp" running.conf
cat running.conf | config-slicer slice --match "system||ntp"
config-slicer slice --match "system||ntp" --vendor junos running.conf
config-slicer slice --match "system||ntp" --json running.conf
```

### Diff two configurations

`diff` applies the same match expression to both inputs before generating a
unified diff.

```bash
config-slicer diff --match "system||ntp" --source before.conf --target after.conf
```

## Match expressions

- Use `||` to separate hierarchy levels.
- Use `*` to match any value at a level.
- Any other level is compiled as a Rust `regex`.

Examples:

- `system||ntp`
- `interfaces||ge-.*||unit||0`
- `routing-options||*`

## Vendor handling

- `autodetect`: choose `junos` when brace-delimited blocks are present, otherwise
  treat the input as `flat`.
- `junos`: brace-delimited hierarchy.
- `flat`: line-oriented configuration such as `set` commands.
