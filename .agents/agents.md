# agents

One-pass orientation docs for LMs working on this codebase. Each memory
covers one concern, meant to be read once at the start of a session,
not re-derived from the code each time.

- `memories/protocol.md` - wire format, opcodes, request/response parsing
- `memories/server.md` - server-side state (windows, clients, resources, ...)
- `memories/display.md` - virtual display rendering and its own thread
- `memories/transport.md` - TCP/Unix socket connection handling
- `memories/extensions.md` - how BIG-REQUESTS/RANDR/etc. are negotiated
- `tasks/implement_opcode/task.md` - step-by-step workflow for adding a
  missing core opcode (parser + handler)
- `tasks/implement_shape_extension/task.md` - next handoff task: SHAPE
  extension is reserved but unimplemented, xeyes needs `ShapeMask`

If something here goes stale, fix the doc in the same commit as the
code change. A wrong doc is worse than no doc.
