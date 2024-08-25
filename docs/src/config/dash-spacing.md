# `dashSpacing`

Control the whitespace behavior of block compact map in block sequence value.
This option is only effective when `indentWidth` is greater than 2.

Possible options:

- `"oneSpace"`: Insert only one space after `-`.
- `"indent"`: Insert spaces to align indentation, respecting `indentWidth` option.

Default option is `"oneSpace"`.

The examples below assume `indentWidth` is `4`.

## Example for `"oneSpace"`

```yaml
outer:
    - key1: value1
      key2: value2
```

## Example for `"indent"`

```yaml
outer:
    -   key1: value1
        key2: value2
```
