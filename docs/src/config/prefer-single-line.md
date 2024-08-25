# `preferSingleLine`

Control whether items should be placed on single line as possible, even they're originally on multiple lines.

Default option value is `false`.

This global option can be overridden by different syntax nodes:

- `flowSequence.preferSingleLine`
- `flowMap.preferSingleLine`

## Example for `false`

```yaml
- [1,
2,
3]

- [
    1, 2, 3
]

- {k1: v1,
    k2: v2,
    k3: v3}

- {
    k1: v1, k2: v2, k3: v3}
```

will be formatted as:

```yaml
- [1, 2, 3]

- [
    1,
    2,
    3,
  ]

- { k1: v1, k2: v2, k3: v3 }

- {
    k1: v1,
    k2: v2,
    k3: v3,
  }
```

## Example for `true`

```yaml
- [1,
2,
3]

- [
    1, 2, 3
]

- {k1: v1,
    k2: v2,
    k3: v3}

- {
    k1: v1, k2: v2, k3: v3}
```

will be formatted as:

```yaml
- [1, 2, 3]

- [1, 2, 3]

- { k1: v1, k2: v2, k3: v3 }

- { k1: v1, k2: v2, k3: v3 }
```
