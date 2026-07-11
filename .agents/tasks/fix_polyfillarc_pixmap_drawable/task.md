# Task: fix PolyFillArc pixmap drawables

## Goal

Fix `PolyFillArc` / opcode 71 so it accepts xproto `DRAWABLE` targets correctly: either a `WINDOW` or a `PIXMAP`.

This is the next narrow compatibility task after `ChangeGC`.

## Current Trace

Live `xeyes` verification now gets past:

- `CreatePixmap`
- `CreateGC`
- `PolyFillRectangle` against a pixmap
- `ChangeGC`

It then fails here:

```text
Dispatching request with opcode: 71 (PolyFillArc)
Handler returned error: Protocol error: FillArc: window 4194319 does not exist
```

The drawable ID exists as a pixmap, but `FillArcHandler` only calls `get_window()`.

## Relevant Files

- `src/protocol/handlers.rs`
- `src/server/graphics.rs`
- `src/server/pixmap_system.rs`

## Important Context

- xproto `DRAWABLE` is a union of `WINDOW` and `PIXMAP`.
- `PolyFillRectangleHandler` was already fixed to resolve the drawable by trying `get_window()` and then `get_pixmap()`.
- `graphics.rs` already has a `Drawable` trait implemented for `Window` and `Pixmap`.
- `fill_rectangle` already uses `&mut impl Drawable`.
- `fill_arc` still takes `&mut Window`, so it cannot draw into a pixmap yet.

## Suggested Implementation

1. Change `fill_arc` in `src/server/graphics.rs` from `&mut Window` to `&mut impl Drawable`, mirroring `fill_rectangle`.
2. Update `FillArcHandler` in `src/protocol/handlers.rs` to resolve `fill_arc_request.drawable` against both windows and pixmaps.
3. Preserve ownership checks:
   - window owner: `window.owner == Some(client_id)`
   - pixmap owner: `pixmap.owner == client_id`
4. Fetch GC foreground as today.
5. Match on the resolved target and call `fill_arc` for each arc on either `get_window_mut()` or `get_pixmap_mut()`.

Keep this task scoped to `PolyFillArc` only. Do not fix `PolyArc`, `PolyLine`, `CopyArea`, or other drawable bugs in the same pass unless explicitly requested.

## Verification

Run:

```sh
cargo build
cargo test --lib
```

Then live verify with Cygwin `xeyes`:

```sh
./target/debug/rxserver.exe
"D:/cygwin64/bin/bash.exe" --login -c "DISPLAY=127.0.0.1:0 /usr/bin/xeyes > /tmp/out.log 2>&1 &"
```

Read `server_stdout.log` / trace logs. Expected result for this task:

- opcode 71 (`PolyFillArc`) dispatches successfully
- `FillArc` returns no response
- stop at the next new failure, likely an unimplemented SHAPE request or another drawable-specific handler

## Worktree Note

At the time this task was written, `rxserver.toml` had an unrelated local change (`debug_overlay = true`) and `.claude/` was untracked. Do not include either unless explicitly asked.

## Recent Commits

- `80b23ce feat(protocol): implement ChangeGC`
- `e283417 style: apply linter formatting`
