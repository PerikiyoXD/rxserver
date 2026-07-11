# server

`src/server/`. Server-side state, one `Server` behind
`Arc<Mutex<Server>>`, shared across all client connections.

## Systems

`Server` composes several small systems, each owning one concern:

- `atom_system` - atom name <-> id table (real intern table, has
  predefined X11 atoms + dynamic ones)
- `window_system` - window hierarchy, pixel data, per-window
  properties (`Window::properties`, used by ChangeProperty/GetProperty)
- `pixmap_system`, `gcontext_system` - pixmaps and graphics contexts
- `client_system` - connected clients, resource ID ownership
- `resource_system` - resource ID allocation bookkeeping
- `display_system` - talks to the display backend(s), see `memories/display.md`
- `extensions: ExtensionRegistry` - see `memories/extensions.md`

Each system is a plain struct with its own methods; `Server` mostly
delegates (`server.get_window(id)` etc). Add new server-visible state
as a new system, not by growing `Server` directly.

## Testing changes here: use a real client, not just cargo build

Protocol/server bugs in this codebase have consistently been things
that compile fine and are wrong at runtime (opcode drift, silent
misrouting, a handler that never sends its reply). `cargo test` covers
unit-level logic (e.g. the property store); it does not catch wire
protocol bugs.

To actually verify a change, use the scripts in `scripts/` (PowerShell)
rather than driving this by hand - they build, launch, PID-track, and
tear down both sides:

```powershell
./scripts/run_server.ps1          # cargo build + launch, RUST_LOG=trace by default
./scripts/is_server_running.ps1   # exit 0 + PID if up, exit 1 otherwise
./scripts/run_xeyes.ps1           # Cygwin xeyes against DISPLAY=127.0.0.1:0
./scripts/is_xeyes_running.ps1
./scripts/stop_xeyes.ps1
./scripts/stop_server.ps1
```

Trace log: `scripts/server_trace.log`. xeyes' own stdout/stderr:
`scripts/xeyes.log`. PID files (`scripts/.server.pid`,
`scripts/.xeyes.pid`) are gitignored - safe to delete if stale.
`run_xeyes.ps1` throws if xeyes exits immediately after launch (it
gets killed by the server hitting an unimplemented request) rather
than hanging - that's a real signal, not a script bug, so check the
log it points at.

Equivalent by hand, if you need to deviate from the scripts (Git
Bash's `/cygdrive/...` paths do not work here - invoke Cygwin's own
bash):

```sh
cargo build
./target/debug/rxserver.exe   # reads rxserver.toml, binds :6000
"D:/cygwin64/bin/bash.exe" --login -c "DISPLAY=127.0.0.1:0 /usr/bin/xeyes > /tmp/out.log 2>&1 &"
```

Read the server's trace-level logs - they show exact opcode dispatch
per request and are how every real bug this codebase has had was
actually found.

**Read the trace from the top, not just the tail.** A disconnect was
chased across multiple sessions (under two different names -
"XInputExtension disconnect", then "WM_PROTOCOLS disconnect") by
repeatedly reading only the last ~30-80 lines around the connection-
close log lines. The real cause - a genuine `ERROR ... Unknown RENDER
minor opcode: 10` parse failure - was sitting hundreds of lines
earlier in the same trace the whole time; reading top to bottom in one
pass found it immediately. The tail shows where the connection dies,
not why - `grep -n "ERROR"` across the *whole* file first, then read
around whatever it finds, before falling back to theorizing about
socket races or client-side toolkit behavior.

`trace_decode.rs` (`protocol::trace_decode::describe_outgoing`) adds a
human-readable one-line decode next to every raw reply/event byte dump
(sequence number, reply length, event code) - check that line instead
of hand-decoding hex when reading a trace.
