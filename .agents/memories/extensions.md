# extensions

X11 extensions (BIG-REQUESTS, RANDR, SHAPE, MIT-SHM, XINERAMA, RENDER)
are handled through `protocol::extension_registry::ExtensionRegistry`,
built once in `Server::new()`.

## What's real vs. what's just a reserved number

All 6 known extensions get a major opcode assigned
(`KNOWN_EXTENSIONS` in `extension_registry.rs`, sequential from
`FIRST_EXTENSION_OPCODE = 128`). `BIG-REQUESTS`, `RANDR`, and `RENDER`
are in `IMPLEMENTED_EXTENSIONS` - they have real parsers and handlers.
`SHAPE`, `MIT-SHM`, and `XINERAMA` are still reserved-only.
`QueryExtensionHandler` reports `present=1` only for implemented ones;
the reserved-only ones get an honest `present=0` even though they
technically have an opcode assigned. Do not flip that without also
writing the parser/handler - a client that believes `present=1` and
sends a real request for an unimplemented extension gets silently
dropped.

RENDER is implemented incrementally, one minor opcode at a time, same
as core opcodes (`tasks/implement_opcode/task.md`) - it currently only
handles `RenderQueryVersion` (0) and `RenderCreateSolidFill` (33), the
two xeyes actually calls. `RenderOpcode` in `types.rs` only has variants
for minor opcodes that are actually parsed; check there (and
`X11RequestParser::parse_dynamic`'s `Some("RENDER") =>` branch) before
assuming a RENDER request is unimplemented.

## Why dynamic and not a fixed number

Real X servers assign extension major opcodes per session; nothing in
the spec guarantees a fixed number for a given extension name.
Hardcoding one number per extension in multiple places (handler
registration, QueryExtension's reply, the parser dispatch) is exactly
how a numbering bug happened here once already. `BigRequestsHandler`
and the RANDR handlers hold their assigned major opcode as a
constructor argument (`Handler::new(major_opcode)`), not a constant.

## Adding a new implemented extension

1. Confirm/add it to `KNOWN_EXTENSIONS` in `extension_registry.rs`.
2. Add it to `IMPLEMENTED_EXTENSIONS` once you actually write handlers
   for it.
3. Give each handler struct a `major_opcode: u8` field + `new()`,
   register it in `create_standard_handler_registry()` behind
   `if let Some(major) = extensions.major_opcode("NAME") { ... }`.
4. Add a branch in `X11RequestParser::parse_dynamic()` that routes
   `extensions.extension_for_opcode(opcode)` to your parser(s).
