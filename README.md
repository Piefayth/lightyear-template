# Lightyear Template

![ci status](https://github.com/piefayth/lightyear-template/actions/workflows/ci.yaml/badge.svg)

An opinionated lightyear starter project.

## Features & Opinions
- Defaults to 3d + leafwing + predicted player + interpolated remotes
- Launcher driven by .ron settings files
- Client that can start and stop hosting (in a second app / second thread) while the app is running.
- Simple zero-dependency asset abstraction for arranging loading states around groups of assets
    - Unified asset folder for client/server
    - Includes an opportunity for asset postprocessing, to add, as this template does, resources like Colliders to entities in converted GLTF Scenes
- Distinct Local/Networked inputs
- WASM setup
- Basic Visual and Networked interpolation setups
- Main and ingame menus for connect/disconnect
- Github Actions CI configurations
    - Test/Clippy/Lint/Docker
    - Release web client to github pages, all platforms to itch.io

## Usage

Replace all instances of `mygame` in the names of folders and files with the name of your game. (or not!)

```
cargo run client -c 1
cargo run server
```

## Crates

### client

### server

### assets
Preloads assets during a managed loading state. Allows for postprocessing loaded GLTFs. Example adds colliders to loaded GLTF.

### common 
Contains all shared gameplay logic between client and server.

### launcher
Non-bevy management code for configuring the client and server apps before running them. Supports native and wasm.

### protocol
Lightyear protocol code. Separate from `common` to save on compile time.

### renders
Shared logic between client and headed server. Anything that the headless server can't run goes here.


## Configuration

Configuration can be modified in `crates/mygame-launcher/options` and extended in `crates/mygame-launcher/launch_options.rs`.

## Running the WASM client

Must modify `crates/mygame-launcher/options/web_client_options.ron` to include the certificate digest for the certs specified in `server_options.ron`.

Install trunk [here](https://trunkrs.dev/) or via `cargo install --locked trunk`.

```
trunk --config ./crates/mygame-launcher/Trunk.toml serve
```

Navigate to `127.0.0.1:8080?client_id=42`

## Certificate instructions for development

From the template root...

```
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -keyout ./crates/mygame-launcher/web/certs/key.pem -out ./crates/mygame-launcher/web/certs/cert.pem -days 14 -nodes -subj "/CN=localhost" -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

## Notes

- `HostServer` mode - client and server in the same `App` - is unsupported. When launching the client and server on the same machine, the server will be launched in its own `App` on a separate thread.
