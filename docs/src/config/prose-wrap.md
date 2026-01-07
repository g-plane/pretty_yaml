# `proseWrap`

Control whether to wrap prose in plain scalars when they exceed the print width.

Possible values:

- `"preserve"`: Don't change how prose is wrapped.
- `"always"`: Wrap prose if it exceeds the print width.

Default value is `"preserve"`.

## Example for `"preserve"`

```yaml
description: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec ac ullamcorper turpis. Curabitur facilisis ut libero a varius. Vivamus nec diam volutpat, semper augue a, semper sapien. Integer ornare ut.
```

## Example for `"always"`

```yaml
description: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec ac
  ullamcorper turpis. Curabitur facilisis ut libero a varius. Vivamus nec diam
  volutpat, semper augue a, semper sapien. Integer ornare ut.
```
