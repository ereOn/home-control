# Home-Control

A home automation control-center for embedded devices.

This software is designed to run on a raspberry pi connected with a touchscreen,
and provides a local web-server that can integrate with an already existing Home
Assistant instance on the local network.

## Cross-compilation

To be able to cross compile (see `scripts/deploy.sh`), you must install some dependencies first:

```bash
sudo apt-get install gcc-arm-none-linux-gnueabihf
rustup target add armv7-unknown-linux-gnueabihf
```

Then run:

```bash
./script/deploy.sh <hostname>
```