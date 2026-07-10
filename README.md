# rxserver

An X11 display server implemented in Rust.

It listens for X11 client connections over TCP and renders windows to
a virtual display (a native OS window, via `softbuffer`). Still under
active development. Many core X11 requests are implemented, others
are not yet.

## Building

```sh
cargo build --release
```

## Running

```sh
cargo run --release
```

By default this reads `rxserver.toml` from the current directory and
listens on `0.0.0.0:6000`. Point an X11 client at it with
`DISPLAY=<host>:0`.

## Configuration

See `rxserver.toml` for logging, transport, and display options.

## License

Apache-2.0

---

If you're a company using rxserver, consider a small donation to
support development: Patreon link coming soon.
