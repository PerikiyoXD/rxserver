# extensions

X11 extensions (BIG-REQUESTS, RANDR, SHAPE, MIT-SHM, XINERAMA, RENDER,
XKEYBOARD, XInputExtension) are handled through
`protocol::extension_registry::ExtensionRegistry`, built once in
`Server::new()`.

## What's real vs. what's just a reserved number

All 8 known extensions get a major opcode assigned
(`KNOWN_EXTENSIONS` in `extension_registry.rs`, sequential from
`FIRST_EXTENSION_OPCODE = 128`). `BIG-REQUESTS`, `RANDR`, `RENDER`,
`SHAPE`, `XKEYBOARD`, and `XInputExtension` are in
`IMPLEMENTED_EXTENSIONS` - they have real parsers and handlers for at
least one request. `MIT-SHM` and `XINERAMA` are still reserved-only.
`QueryExtensionHandler` reports `present=1` only for implemented ones;
the reserved-only ones get an honest `present=0` even though they
technically have an opcode assigned. Do not flip that without also
writing the parser/handler - a client that believes `present=1` and
sends a real request for an unimplemented extension gets silently
dropped.

XKEYBOARD only has `XkbUseExtension` (minor opcode 0) implemented -
the version-negotiation handshake, and the only XKB request any live
client has been observed to send (xeyes/Xt sends it once immediately
after `QueryExtension` reports `present=1`, then never sends another
XKB request for the rest of the connection). `XkbOpcode` in
`types.rs` lists the *full* XKB minor-opcode set from the protocol
spec (Bell, GetState, GetMap, SetMap, GetControls, GetNames, indicator
requests, per-device info, ~23 opcodes total) purely so dispatch/trace
logging can name them if they ever show up - none of the rest have
real handler logic, and none should be built speculatively. If a live
trace ever shows a client sending one of them, implement that one
opcode, verify live, stop - same discipline as RENDER/SHAPE below.

XInputExtension has three requests implemented, all confirmed live
against xeyes/Xt in the order it actually calls them:
`GetExtensionVersion` (XI1 minor opcode 1, replies `present=0` - no
device subsystem backs it), `XIQueryVersion` (XI2 minor opcode 47,
replies with version 2.0 - unlike `GetExtensionVersion`'s reply, XI2's
has no `present` flag, so *not* replying is not an option: a live
trace confirmed Xt blocks waiting for this specific reply and hangs
the connection for 30s before timing out if it never arrives), and
`XISelectEvents` (XI2 minor opcode 46, no reply per spec - stores
per-device event masks on `Window::xi_event_masks`, which nothing
reads yet since no XI2 event this server can generate exists). This
resolved the multi-session "XInputExtension disconnect" investigation
- xeyes/Xt now completes the full negotiation chain
(`QueryExtension` -> `GetExtensionVersion` -> `XIQueryVersion` ->
`XISelectEvents`) without error. A *different*, older disconnect
remains after this point (client closes its socket right after
`InternAtom(WM_PROTOCOLS)`) - see
`tasks/investigate_wm_protocols_disconnect/task.md`.

`XInputOpcode` in `types.rs` lists the full XI1 (1-35) and the one
known XI2 (46, 47) minor opcode set for naming, same
"list everything, implement only what's confirmed" pattern as
`XkbOpcode`. XI2 has far more than 2 opcodes in the real spec (~20) -
only 46 and 47 are listed here because those are the only two ever
observed in a live trace; add more only when a trace shows them.

RENDER and SHAPE are both implemented incrementally, one minor opcode
at a time, same as core opcodes (`tasks/implement_opcode/task.md`) -
add exactly the minor opcode a live xeyes trace actually calls next,
verify, stop. Don't pre-build the rest of an extension's requests on
spec alone.

RENDER currently handles `RenderQueryVersion` (0),
`RenderQueryPictFormats` (1), `RenderCreatePicture` (4), and
`RenderCreateSolidFill` (33). `RenderQueryPictFormats` replies with one
hardcoded PictFormat (depth-24 Direct/TrueColor) matching the single
visual `connection.rs`'s connection-setup reply advertises (visual id
`0x21`, masks `0xFF0000`/`0x00FF00`/`0x0000FF`, no alpha) - if the
server ever grows more than one visual/depth/screen, that reply needs
to grow with it instead of staying hardcoded.
`RenderCreatePicture` supports the full CreatePicture value-mask
(`picture_value_mask` in `handlers.rs`, mirrors `gc_value_mask`'s
apply-if-set-bit pattern) via `PictureAttributes` in
`picture_system.rs`; a Picture backed by a real drawable is
`PictureContent::Drawable`, resolved against window-or-pixmap the same
way `PolyFillRectangleHandler` resolves a `DRAWABLE`.

SHAPE currently handles only `ShapeMask` (2) - `ShapeQueryVersion` (0)
is *not* implemented despite having an `Opcode` variant reserved for
it in `types.rs`; xeyes skips straight to `ShapeMask` without querying
version first, so nothing has exercised opcode 0 yet. The bounding
shape itself is stored as `Window::bounding_shape: Option<PixmapId>` -
deliberately minimal, no region/rectangle-list math
(`ShapeRectangles`, `ShapeCombine`, `ShapeOffset` are unimplemented).

`RenderOpcode`/`ShapeOpcode` in `types.rs` only have variants for
minor opcodes that are actually parsed; check there (and
`X11RequestParser::parse_dynamic`'s `Some("RENDER")`/`Some("SHAPE")`
branches) before assuming a request is unimplemented. `XkbOpcode` is
the deliberate exception - it lists every real XKB minor opcode for
naming purposes even though only `UseExtension` has a parser; check
`parse_dynamic`'s `Some("XKEYBOARD")` branch (not the enum) for what's
actually dispatchable.

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
