# corematch

Corematch is a memory game where the player has to match the latest Polkadot cores usage in a 4x4 matrix. Corematch is written in Rust and compiled to WASM to run entirely in the browser (Subxt + Yew)






## Building the package

```
wasm-pack build --target web
```

```
cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --target web"
```