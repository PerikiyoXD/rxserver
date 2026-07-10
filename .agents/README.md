# .agents

One-pass orientation docs for LMs working on this codebase. Each file
covers one concern, meant to be read once at the start of a session,
not re-derived from the code each time.

- `protocol.md` - wire format, opcodes, request/response parsing
- `server.md` - server-side state (windows, clients, resources, ...)
- `display.md` - virtual display rendering and its own thread
- `transport.md` - TCP/Unix socket connection handling
- `extensions.md` - how BIG-REQUESTS/RANDR/etc. are negotiated

If something here goes stale, fix the doc in the same commit as the
code change. A wrong doc is worse than no doc.
