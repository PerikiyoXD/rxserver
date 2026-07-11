# Task: decide how to handle XInputExtension

## Goal

Live `xeyes` verification (`memories/server.md`) now gets all the way
through core protocol, `BIG-REQUESTS`, RENDER (`QueryVersion`,
`QueryPictFormats`, `CreatePicture`), SHAPE's `ShapeMask`,
`MapSubwindows` (opcode 9), and `MapWindow` (opcode 8) - every request
the client sends gets a correct handler response. Xlib then does a
`QueryExtension` for `XInputExtension`, gets back `present=0` (it is
not in `KNOWN_EXTENSIONS` at all, unlike SHAPE/MIT-SHM/XINERAMA which
are at least reserved), and immediately closes the connection
client-side:

```text
Xlib:  extension "XInputExtension" missing on display "127.0.0.1:0".
X connection to 127.0.0.1:0 broken (explicit kill or server shutdown).
```

This races with the server's in-flight write of the *next* request's
reply (`InternAtom` for `WM_PROTOCOLS`), which logs as `Failed to send
response` - that error is a downstream symptom of the client already
having hung up, not a separate bug to chase.

## Notes

- First confirm whether XInputExtension is actually required for
  `xeyes` to keep running, or whether this is Xlib's own
  belt-and-suspenders probe that a real X server also fails and Xlib
  tolerates in some configurations but not others (worth checking
  against a real Xorg/Xvfb trace if available, or the Xlib source's
  handling of a missing XInputExtension on `XOpenDisplay`).
- If it does need to be implemented: XInputExtension (XI/XI2) is a
  large, versioned extension (device enumeration, event masks per
  device). Do not build it out - same incremental-per-minor-opcode
  discipline as RENDER/SHAPE
  (`memories/extensions.md`/`tasks/implement_opcode/task.md`): find out
  exactly what minimal response makes Xlib's probe succeed (likely just
  `QueryExtension` reporting `present=1` plus whatever XInputExtension
  opcode 0 - `XIQueryVersion` or similar - Xlib calls right after) and
  implement only that.
- If it turns out xeyes doesn't strictly need it (e.g. only breaks
  because this server's `QueryExtension` answers "not present" instead
  of something Xlib tolerates more gracefully), the fix might be
  narrower than a full extension - check what a real server returns for
  `present=0` on this extension and whether Xlib's disconnect is
  actually specific to *how* absence is reported.
- Verify live per `memories/server.md`. Stop at whatever the next
  failure is - report it, don't chase it in the same pass.
