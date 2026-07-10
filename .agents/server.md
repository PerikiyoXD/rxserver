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
- `display_system` - talks to the display backend(s), see `display.md`
- `extensions: ExtensionRegistry` - see `extensions.md`

Each system is a plain struct with its own methods; `Server` mostly
delegates (`server.get_window(id)` etc). Add new server-visible state
as a new system, not by growing `Server` directly.

## Testing changes here: use a real client, not just cargo build

Protocol/server bugs in this codebase have consistently been things
that compile fine and are wrong at runtime (opcode drift, silent
misrouting, a handler that never sends its reply). `cargo test` covers
unit-level logic (e.g. the property store); it does not catch wire
protocol bugs.

To actually verify a change:

```sh
cargo build
./target/debug/rxserver.exe   # reads rxserver.toml, binds :6000
```

Then connect a real X11 client. On this dev machine that means Cygwin
(Git Bash's `/cygdrive/...` paths do not work here - invoke Cygwin's
own bash):

```sh
"D:/cygwin64/bin/bash.exe" --login -c "DISPLAY=127.0.0.1:0 /usr/bin/xeyes > /tmp/out.log 2>&1 &"
```

Read the server's trace-level logs - they show exact opcode dispatch
per request and are how every real bug this codebase has had was
actually found.
