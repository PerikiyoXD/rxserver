# protocol

`src/protocol/`. Wire format, opcode numbers, request parsing, request
handling.

## Opcode numbers: one source of truth

`protocol::types::Opcode` is the only place core opcode numbers (1-127)
live (`Opcode::to_u8()` / `Opcode::from_u8()`). Do not hardcode a
number for a core request anywhere else. This used to be duplicated
in a separate `opcodes` module and drifted out of sync silently
(`CopyArea`/`PolyArc`/`FillArc` were wrong for a while), which
misrouted real client requests without any compile error.

RANDR minor opcodes live in `protocol::types::RandrOpcode` the same
way.

## Extension major opcodes are dynamic, not fixed

Real X servers assign extension major opcodes (BIG-REQUESTS, RANDR,
etc.) per session; clients must call `QueryExtension` to learn them,
never assume a fixed number. This server does the same via
`protocol::extension_registry::ExtensionRegistry`, built once in
`Server::new()`. See `extensions.md`.

## Dispatch

- `X11RequestParser::parse(bytes)` - core opcodes only (< 128,
  `FIRST_EXTENSION_OPCODE` in `protocol/constants.rs`). No server
  state needed.
- `X11RequestParser::parse_dynamic(bytes, &ExtensionRegistry)` - the
  real entry point, used everywhere requests actually come in
  (`server::connection`). Falls back to `parse()` for core opcodes,
  resolves extension opcodes against the registry.

Each request type has a `FooParser` (implements `RequestParser`,
parses raw bytes into a `RequestKind::Foo(FooRequest)`) and a
`FooHandler` (implements `RequestHandler`, does the actual work).
Parsers live in `request.rs`, handlers in `handlers.rs` (RANDR
handlers in `randr/handlers.rs`).

## RequestHandlerRegistry keys on (major, minor)

`RequestHandler::opcode()` returns `(u8, Option<u8>)`, not just `u8`.
Core requests use `(N, None)`. Extension sub-requests (e.g. every
RANDR request) share one major opcode and are distinguished by minor,
so they need the pair. Do not encode major+minor as a single fake
number into `Request.opcode` to work around this - that happened once
already and was a real structural bug, not a shortcut.

## Adding a new request type

1. Add/confirm the opcode on `Opcode` (types.rs) if it's core, or
   register the extension in `extension_registry.rs` if not.
2. Add a `FooRequest` struct + `FooParser` in `request.rs`, wire it
   into `RequestKind`, `parse()`/`parse_dynamic()`, and `validate()`.
3. Add a `FooHandler` in `handlers.rs`, register it in
   `create_standard_handler_registry()`.
4. Test against a real client, not just `cargo build`. See
   `server.md` for how.
