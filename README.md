# corematch

<p align="center">
  <img src="https://github.com/turboflakes/corematch/blob/main/corematch_github_header.png?raw=true">
</p>

Corematch ― Unstopabble memory game where players match the latest Polkadot core usage in a 3x3 matrix board. Corematch is written in Rust and compiled to WASM to run entirely in the browser ([Subxt](https://github.com/paritytech/subxt) + [Yew](https://yew.rs/))

## ✨ Included Features

- [&check;] Support Polkadot and Kusama network;
- [&check;] Mobile first support;
- [&check;] Play with keyboard, mouse or touch;
- [&check;] Two challenging game levels;
- [&check;] Optional help which highlights matches;

## 🚧 Work In Progress

- [] Mint points as NFT on AssetHub;
- [] Light clients support;
- [] Signing via PJS extension and other wallets;

## Development / Build from Source

If you'd like to build from source, first install Rust.

```bash
#!/bin/bash
curl https://sh.rustup.rs -sSf | sh
```

If Rust is already installed run

```bash
#!/bin/bash
rustup update
```

Verify Rust installation by running

```bash
#!/bin/bash
rustc --version
```

Once done, finish installing the support software

```bash
#!/bin/bash
sudo apt install build-essential git clang libclang-dev pkg-config libssl-dev
```

Add WebAssembly target to your development environment

```bash
#!/bin/bash
rustup target add wasm32-unknown-unknown
```

Install Trunk

```bash
#!/bin/bash
cargo install --locked trunk
```

Build `corematch` by cloning this repository

```bash
#!/bin/bash
git clone http://github.com/turboflakes/corematch
```

Finally Use `trunk` to build and serve the app

```bash
#!/bin/bash
trunk serve
```

## Collaboration

Have an idea for a new feature, a fix or you found a bug, please open an [issue](https://github.com/turboflakes/crunch/issues) or submit a [pull request](https://github.com/turboflakes/crunch/pulls).

Any feedback is welcome.

## About

Corematch - was made by **Turboflakes**. Visit us at <a href="https://turboflakes.io" target="_blank" rel="noreferrer">turboflakes.io</a> to know more about our work.

If you like this project
  - 🚀 Share our work 
  - ✌️ Visit us at <a href="https://turboflakes.io" target="_blank" rel="noreferrer">turboflakes.io</a>
  - ✨ Or you could also star the Github project :)

Tips are welcome

- Polkadot 14Sqrs7dk6gmSiuPK7VWGbPmGr4EfESzZBcpT6U15W4ajJRf (turboflakes.io)
- Kusama H1tAQMm3eizGcmpAhL9aA9gR844kZpQfkU7pkmMiLx9jSzE (turboflakes.io)

### License

Corematch - The entire code within this repository is licensed under the [Apache License 2.0](./LICENSE).