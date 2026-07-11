# Task: implement the SHAPE extension

## Goal

`SHAPE` currently has a major opcode reserved in `KNOWN_EXTENSIONS`
but is not in `IMPLEMENTED_EXTENSIONS` - no parser, no handler. xeyes
needs it (that's the whole point of the app: a non-rectangular window
shaped by a mask). This is a new-extension job (`memories/extensions.md`
"Adding a new implemented extension"), not a core-opcode job
(`tasks/implement_opcode/task.md`).

## Current Trace

Live `xeyes` verification now gets past all of core protocol up
through:

- `CreatePixmap` / `CreateGC` / `PolyFillRectangle` (window and pixmap
  drawables)
- `ChangeGC`
- `PolyFillArc` (window and pixmap drawables)

It then fails here:

```text
Dispatching extension request with opcode: 130 (UnknownExtension(130, 0))
Failed to parse request from client 1:
Extension 'SHAPE' has an assigned opcode but no request parser yet
```

The raw bytes were `[130, 2, 5, 0, 0, 0, 64, 0, 13, 0, 64, 0, 0, 0, 0, 0, 15, 0, 64, 0]`
- major 130 (SHAPE), **minor 2**, not 0. Same pattern as RENDER: xeyes
skips `ShapeQueryVersion` and calls a real request first.

## Wire Format

Source of truth: `D:\code\X11\xorgproto\include\X11\extensions\shapeproto.h`
(there is no `shapeproto.txt` in this checkout - use the header's
struct layout directly, field order and sizes are exact). Do not guess
byte layout from the request name.

Minor opcodes seen in that header:

```
0  X_ShapeQueryVersion
1  X_ShapeRectangles
2  X_ShapeMask       <- what xeyes calls first
3  X_ShapeCombine
4  X_ShapeOffset
5  X_ShapeQueryExtents
6  X_ShapeSelectInput
7  X_ShapeInputSelected
8  X_ShapeGetRectangles
```

`X_ShapeMask` (`xShapeMaskReq`, `shapeproto.h` line ~86), 20 bytes:

```
CARD8  reqType       // major opcode (130 this session)
CARD8  shapeReqType  // 2
CARD16 length        // 5 (4-byte units)
CARD8  op            // Set/Union/Intersect/Subtract/Invert
CARD8  destKind      // ShapeBounding or ShapeClip
CARD16 junk          // unused padding
Window dest
INT16  xOff
INT16  yOff
CARD32 src           // a 1-bit-deep pixmap, or None to clear the shape
```

The observed bytes decode as: op=0 (Set), destKind=0 (ShapeBounding),
dest=window 0x00400010, xOff=0, yOff=0, src=pixmap 0x0040000f.

## Relevant Files

Same four places as any new opcode (`tasks/implement_opcode/task.md`
Givens), plus the extension registration steps
(`memories/extensions.md`):

- `src/protocol/types.rs` - add `ShapeOpcode` enum (minor opcodes only;
  major is session-dynamic, same as `RenderOpcode` - see the RENDER
  precedent, do not add a fixed `MAJOR_OPCODE` constant)
- `src/protocol/request.rs` - request struct + parser, wired into
  `RequestKind`, `X11RequestParser::parse_dynamic()`'s `Some("SHAPE") =>`
  branch (new), and `validate()`
- `src/protocol/handlers.rs` - handler, registered behind
  `if let Some(major) = extensions.major_opcode("SHAPE")`
- `src/protocol/extension_registry.rs` - add `"SHAPE"` to
  `IMPLEMENTED_EXTENSIONS`

## Server State Needed

There is currently no concept of a window's shape mask anywhere in
`window_system.rs`. `ShapeMask` needs somewhere to store it - a
minimal `Option<PixmapId>` (or similar) field on `Window` for the
bounding-shape pixmap is probably enough for what xeyes needs; don't
build out full region/rectangle-list shape support (`ShapeRectangles`,
`ShapeCombine`, region math) unless a later trace actually calls for
it. Follow `colormap_system.rs`'s "minimal new system" shape per
`tasks/implement_opcode/task.md`'s Givens.

## Verification

```sh
cargo build
cargo test --lib
```

Then live verify with Cygwin `xeyes` (see `memories/server.md`):

```sh
./target/debug/rxserver.exe
"D:/cygwin64/bin/bash.exe" --login -c "DISPLAY=127.0.0.1:0 /usr/bin/xeyes > /tmp/out.log 2>&1 &"
```

Read the trace log. Expected result for this task:

- `QueryExtension` for `'SHAPE'` reports `present=1`
- opcode 130 minor 2 (`ShapeMask`) dispatches and returns successfully
- stop at whatever xeyes calls next - do not implement other SHAPE
  minor opcodes (`ShapeRectangles`, `ShapeCombine`, etc.) or continue
  chasing further failures in the same pass unless explicitly asked

## Worktree Note

At the time this task was written, `rxserver.toml` had an unrelated
local change (`debug_overlay = true`) and `.claude/` was untracked.
Do not include either unless explicitly asked.

## Recent Commits

- `66cf949 fix(protocol): resolve PolyFillArc drawable against pixmaps too`
- `d9b253b fix(protocol): resolve PolyFillRectangle drawable against pixmaps too`
- `80b23ce feat(protocol): implement ChangeGC`
