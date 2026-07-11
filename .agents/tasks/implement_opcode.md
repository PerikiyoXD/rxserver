# Workflow: implement a missing core opcode

Used when a real client (xeyes via Cygwin, see `server.md`) dies with
`Unknown opcode: N` - the opcode has a name in `protocol::types::Opcode`
(or needs one added) but no parser/handler exists yet. This is how
QueryColors (91), FreePixmap (54), and GetInputFocus (43) were each
added, one opcode per pass.

## Givens (true every time)

- The opcode number is core protocol (< 128) unless the trace shows
  `Dispatching extension request` - extension opcodes are a different,
  bigger job (see `extensions.md`), not this workflow.
- `Opcode::to_u8()`/`from_u8()`/`name()` in `types.rs` may already have
  the variant even though nothing parses it - check before adding one,
  since it's easy to assume the opcode is entirely missing when only
  the parser/handler are.
- Request struct, parser, and handler each live in a fixed place:
  request struct + `FooParser` in `request.rs`, `FooHandler` in
  `handlers.rs`. Never split this across other files.
- Every request parser is wired in *four* places in `request.rs`, all
  required, and every implemented opcode has all four instances (grep
  for a request you know works, e.g. `GetProperty`, to see the shape):
  1. `RequestKind` enum variant
  2. parser struct declaration (`pub struct FooParser;`)
  3. `impl RequestParser for FooParser` (parse + validate)
  4. dispatch in `X11RequestParser::parse()`'s `if/else if` chain, and
     a matching arm in `X11RequestParser::validate()`
- Handlers need: the handler struct, `impl RequestHandler`
  (`handle_request`, `opcode()`, `name()`), and a
  `registry.register_handler(FooHandler)` line in
  `create_standard_handler_registry()` (`handlers.rs`). Forgetting the
  registry line compiles fine and silently drops the request at
  runtime - this has been the single most common mistake.
- Don't guess the wire format. Byte layout (field order, sizes, which
  fields are padding) comes from the real X11 protocol spec for that
  request, not from pattern-matching a similarly-named one.
- If the request needs server state that doesn't exist yet (a
  colormap system, focus tracking, etc.), adding a minimal version of
  that state is expected, not scope creep - see `colormap_system.rs`
  for the shape of a new system (struct + `new()` + accessor methods,
  wired into `Server` in `state.rs` and `mod.rs`). Keep it to exactly
  what the opcode needs; don't build out the whole subsystem.

## Steps

1. Find or add the `Opcode` variant in `types.rs` (all three mapping
   functions: `from_u8`, `to_u8`, `name`).
2. Add the request struct + parser in `request.rs`, following the
   nearest existing request of the same shape (header-only like
   `NoOperation`/`GetInputFocus`, fixed-fields like `FreeGC`/
   `FreePixmap`, or variable-length like `GetProperty`/`QueryColors`).
   Wire all four places listed above.
3. Add the handler in `handlers.rs`, register it in
   `create_standard_handler_registry()`.
4. `cargo build`.
5. Verify live per `server.md`: launch `rxserver.exe` with
   `RUST_LOG=trace`, launch xeyes via Cygwin, read the trace log for
   the opcode dispatching, the handler's reply being written, and
   where the client goes *next*. `cargo build` succeeding proves
   nothing about wire correctness on its own.
6. Stop at one opcode. The next `Unknown opcode: N` (or
   `UnknownExtension`) in the trace is the next task, not something to
   chase in the same pass - report exactly where the client got to and
   stop there.
