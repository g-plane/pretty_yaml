# `quotes`

Control the quotes.

Possible options:

- `"preferDouble"`: Use double quotes as possible. However if there're escaped characters in strings, quotes will be kept as-is.
- `"preferSingle"`: Use single quotes as possible. However if there're `\` char or `"` char in strings, quotes will be kept as-is.

Default option is `"preferDouble"`.
We recommend to use double quotes because behavior in single quoted scalars is counter-intuitive.

## Example for `"preferDouble"`

```yaml
- "text"
```

## Example for `"preferSingle"`

```yaml
- 'text'
```
