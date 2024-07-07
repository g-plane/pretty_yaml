# Configuration

Options name in this page are in camel case.
If you're using Pretty YAML as a Rust crate, please use snake case instead.

## `printWidth`

The line width limitation that Pretty YAML should *(but not must)* avoid exceeding. Pretty YAML will try its best to keep line width less than this value, but it may exceed for some cases, for example, a very very long single word.

Default option is `80`.

## `useTabs`

Specify use space or tab for indentation.

Default option is `false`.

## `indentWidth`

Size of indentation. When enabled `useTabs`, this option may be disregarded,
since only one tab will be inserted when indented once.

Default option is `2`. This can't be zero.

## `lineBreak`

Specify use `\n` (LF) or `\r\n` (CRLF) for line break.

Default option is `"lf"`. Possible options are `"lf"` and `"crlf"`.

## `quotes`

Control the quotes.

Possible options:

- `"preferDouble"`: Use double quotes as possible. However if there're escaped characters in strings, quotes will be kept as-is.
- `"preferSingle"`: Use single quotes as possible. However if there're `\` char or `"` char in strings, quotes will be kept as-is.

Default option is `"preferDouble"`.
We recommend to use double quotes because behavior in single quoted scalars is counter-intuitive.

### Example for `"preferDouble"`

```yaml
- "text"
```

### Example for `"preferSingle"`

```yaml
- 'text'
```

## `trailingComma`

Control whether trailing comma should be inserted or not.

Default option is `true`.

### Example for `false`

```yaml
- [
    a
  ]
- {
    a: b
  }
```

### Example for `true`

```yaml
- [
    a,
  ]
- {
    a: b,
  }
```

## `formatComments`

Control whether whitespace should be inserted at the beginning of comments or not.

When this option is set to `false`, comments contain leading whitespace will still be kept as-is.

Default option is `false`.

### Example for `false`

```yaml
#comment
```

### Example for `true`

```yaml
# comment
```

## `indentBlockSequenceInMap`

Control whether block sequence should be indented or not in a block map.

Default option is `true`.

### Example for `false`

```yaml
key:
- item
```

### Example for `true`

```yaml
key:
  - item
```

## `braceSpacing`

Control whether whitespace should be inserted between braces or not.

Default option is `true`.

### Example for `false`

```yaml
{a: b}
```

### Example for `true`

```yaml
{ a: b }
```

## `bracketSpacing`

Control whether whitespace should be inserted between brackets or not.

Default option is `false`.

### Example for `false`

```yaml
[a, b]
```

### Example for `true`

```yaml
[ a, b ]
```

## `dashSpacing`

Control the whitespace behavior of block compact map in block sequence value.
This option is only effective when `indentWidth` is greater than 2.

Possible options:

- `"oneSpace"`: Insert only one space after `-`.
- `"indent"`: Insert spaces to align indentation, respecting `indentWidth` option.

Default option is `"oneSpace"`.

The examples below assume `indentWidth` is `4`.

### Example for `"oneSpace"`

```yaml
outer:
    - key1: value1
      key2: value2
```

### Example for `"indent"`

```yaml
outer:
    -   key1: value1
        key2: value2
```

## `trimTrailingWhitespaces`

Control whether trailing whitespaces should be trimmed or not.

Default option is `true`.

## `ignoreCommentDirective`

Text directive for ignoring formatting specific content.

Default is `"pretty-yaml-ignore"`.
