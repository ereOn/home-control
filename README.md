# Home-Control

A home automation control-center for embedded devices.

This software is designed to run on a raspberry pi connected with a touchscreen,
and provides a local web-server that can integrate with an already existing Home
Assistant instance on the local network.

## Building

The Rust build process will try to embed the generated static web files which
must first be build at least once.

To build, you'll need to have the following installed:

- `cargo` (part of the Rust toolchain)
- `rustup` (part of the Rust toolchain)
- `npm` (part of the Node.js toolchain)

Once  you have the required tools installed, you can run the following command:

```bash
make
```

## Development

Running the binary on the local machine in deployment requires a few additional
things:

- `tmux`
- `cargo watch` (`cargo install cargo-watch`)

Then, set the following environment variables so that `cargo run` can work
without explicit arguments:

- `HOME_ASSISTANT_ENDPOINT`: The `hostname:port` of the Home Assistant instance,
without any protocol scheme (do not include the `http://` or `https://`).
- `HOME_ASSISTANT_TOKEN`: The Home Assistant API long-lived token. You can
generate one from your Home Assistant user profile page. You may want to create
a user with limited permissions to generate the token.

Then run:

```bash
make dev
```

## Cross-compilation and deployment on a Raspberry Pi

To be able to cross compile (see `scripts/deploy.sh`), you must install some dependencies first:

```bash
sudo apt-get install gcc-arm-none-linux-gnueabihf build-essential g++-arm-linux-gnueabihf
rustup target add armv7-unknown-linux-gnueabihf
```

Then run:

```bash
./script/deploy.sh <hostname>
```

Or set the `DEPLOY_TARGET_HOST` to the SSH hostname or IP address of the target Raspberry Pi machine and call:

```bash
make deploy
```