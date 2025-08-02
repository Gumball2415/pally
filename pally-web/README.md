# Pally WASM

Yet another NES palette generator, made with Rust for the web!

## Requirements

- [`wasm-bindgen`](https://github.com/wasm-bindgen/wasm-bindgen)

```sh
cargo install -f wasm-bindgen-cli
```

## Building

to build:

```sh
wasm-pack build --target web
```

due to CORS limitation, you must open `index.html` inside an HTTP server:

```sh
python3 -m http.server 8080
```

you may also use [miniserve](https://crates.io/crates/miniserve):

```sh
miniserve . --index "index.html" -p 8080
```

## Licence

This work is licensed under the MIT license.

Copyright (C) Persune 2025.
