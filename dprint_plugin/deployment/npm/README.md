# dprint-plugin-yaml

npm distribution of the [Pretty YAML](https://github.com/g-plane/pretty_yaml) [dprint](https://dprint.dev/) plugin.

## Install

```shell
npm install --save dprint-plugin-yaml
```

## Usage

Use this package to get the path to the plugin's Wasm module from JavaScript:

```js
import { getPath } from "dprint-plugin-yaml";

// or:
const { getPath } = require("dprint-plugin-yaml");

// absolute path to the Wasm module:
getPath();
```

The Wasm file is also reachable directly via the `dprint-plugin-yaml/plugin.wasm` subpath export.

## Configuring dprint

To use the plugin from `dprint.json` directly, see the instructions in the [main README](https://github.com/g-plane/pretty_yaml#dprint). Configuration options are documented at <https://pretty-yaml.netlify.app/>.

## License

MIT
