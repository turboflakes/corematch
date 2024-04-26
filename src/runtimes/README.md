## Supported Runtimes
  - Polkadot
  - Kusama
  - Asset Hub Polkadot
  - Asset Hub Kusama

## Generated files from subxt-cli

Download metadata from a substrate node, for use with `subxt` codegen.

```bash
subxt metadata --url wss://rpc.turboflakes.io:443/polkadot -f bytes > polkadot_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/kusama -f bytes > kusama_metadata.scale
subxt metadata --url wss://sys.turboflakes.io:443/statemint -f bytes > asset_hub_polkadot_metadata.scale
subxt metadata --url wss://sys.turboflakes.io:443/statemine -f bytes > asset_hub_kusama_metadata.scale
subxt metadata --url wss://sys.turboflakes.io:443/westmint -f bytes > asset_hub_westend_metadata.scale
```

Generate runtime API client code from metadata.

```bash
subxt codegen --url wss://rpc.turboflakes.io:443/polkadot | rustfmt --edition=2018 --emit=stdout > polkadot_runtime.rs
subxt codegen --url wss://rpc.turboflakes.io:443/kusama | rustfmt --edition=2018 --emit=stdout > kusama_runtime.rs
subxt codegen --url wss://sys.turboflakes.io:443/statemint | rustfmt --edition=2018 --emit=stdout > asset_hub_polkadot_runtime.rs
subxt codegen --url wss://sys.turboflakes.io:443/statemine | rustfmt --edition=2018 --emit=stdout > asset_hub_kusama_runtime.rs
subxt codegen --url wss://sys.turboflakes.io:443/westmint | rustfmt --edition=2018 --emit=stdout > asset_hub_westend_runtime.rs
```