# transport

`src/transport/`. Accepts client connections (TCP and, on Unix, a
Unix domain socket), hands each one to `server::connection::ConnectionHandler`.

- `tcp.rs` - real, used by default (`rxserver.toml` transports section,
  default bind `0.0.0.0:6000`).
- `unix.rs` - `#[cfg(unix)]`, never built or tested on this
  (Windows) dev machine. Its `ConnectionHandler::new_unix` constructor
  has a known field-name mismatch bug (see `connection.rs`'s TODO
  comment near it) that has never surfaced because it never compiles
  here. Verify on a real Unix box before trusting this path.

Per-connection request handling (parsing, dispatch, replies) lives in
`server::connection`, not here - this module is purely about accepting
the socket.
