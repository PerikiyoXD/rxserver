# agents

One-pass orientation docs for LMs working on this codebase. Each memory
covers one concern, meant to be read once at the start of a session,
not re-derived from the code each time.

- `memories/protocol.md` - wire format, opcodes, request/response parsing
- `memories/server.md` - server-side state (windows, clients, resources, ...)
- `memories/display.md` - virtual display rendering and its own thread
- `memories/transport.md` - TCP/Unix socket connection handling
- `memories/extensions.md` - how BIG-REQUESTS/RANDR/RENDER/SHAPE are
  negotiated and implemented incrementally
- `tasks/implement_opcode/task.md` - step-by-step workflow for adding a
  missing core opcode (parser + handler)
- `tasks/investigate_wm_protocols_disconnect/task.md` - next handoff
  task: the XInputExtension disconnect is resolved; a different,
  older disconnect remains right after `InternAtom(WM_PROTOCOLS)`

If something here goes stale, fix the doc in the same commit as the
code change. A wrong doc is worse than no doc.

## Handoff prompt

At the end of a work session (task done, or stopped at a verified
stopping point per `tasks/implement_opcode/task.md`'s "stop at this one
opcode" discipline), emit a self-contained handoff prompt so the next
session can resume cold, bracketed exactly like this:

```text
=== I HANDOFF PROMPT ===
<prompt text: what's done, what's verified, where the live trace left
off, what's next, and any pitfalls not to re-litigate - written as
context for an agent with no memory of this session>
=== READY TO HANDOFF ===
```

The bracketed prompt is the deliverable, not prose around it - keep
narration outside the markers minimal.
