# Task: fix the XInputExtension-triggered client disconnect

## What's confirmed (live trace, not guessed)

xeyes now completes far more of its setup than earlier sessions
assumed. As of the XKEYBOARD `UseExtension` work, a live trace shows
xeyes/Xt successfully getting through: BIG-REQUESTS, XKEYBOARD
(`QueryExtension` + `UseExtension`), SHAPE (`ShapeMask`), RENDER
(`QueryVersion`, `QueryPictFormats`, `CreatePicture` x2),
`MapSubwindows` (9), `MapWindow` (8), several `InternAtom`s
(`WM_PROTOCOLS`, `Custom Init`, `Custom Data`, ...), `CreateWindow`,
many `ChangeProperty`s, `PolyFillRectangle`, `PolyFillArc`,
`ClearArea`, `CreatePixmap`/`FreePixmap` cycles, `GetInputFocus`,
`QueryColors` - every one of these gets a correct handler response.

`XInputExtension` is queried via `QueryExtension` (twice), gets an
honest `present=0` both times, and **this is tolerated** - xeyes does
not disconnect at that point and keeps issuing further requests for
some time afterward (confirmed: RENDER/SHAPE/window-setup calls happen
*after* the XInputExtension query in the same trace).

The disconnect happens later: the client sends a final batch of
requests ending in `InternAtom` for `WM_PROTOCOLS`, then closes its
socket before the server's reply write completes
(`scripts/connection.rs`'s `write_all(...).context("Failed to send
response")` fails - a real socket error, not a bug in that code). The
client-side log is always exactly:

```text
Xlib:  extension "XInputExtension" missing on display "127.0.0.1:0".
X connection to 127.0.0.1:0 broken (explicit kill or server shutdown).
```

Working theory (not yet proven): Xt/libXi's `XInputExtension` probe
sets a deferred/fatal flag when it gets `present=0`, but doesn't act
on it immediately - the client keeps pipelining requests it already
queued via other Xlib/Xt calls, then hits the fatal path (likely on
its own next `XSync`/`_XReply`/event-queue check) and tears down the
socket without waiting for the server's last reply. This would explain
why so much real work completes successfully before the disconnect.

## What's not yet done

- Confirm the working theory above against real Xlib/libXi source
  (Cygwin's `cygXi-6.dll`/`cygXt-6.dll`) rather than continuing to
  infer it from trace timing alone.
- Decide the fix: most likely, implementing enough of
  `XInputExtension` for `QueryExtension` to report `present=1` (same
  path as XKEYBOARD this session) so Xt's probe succeeds instead of
  failing. XInput is a much larger extension than XKB - do not
  build more than the minimal request Xt actually sends first
  (probably `GetExtensionVersion`, XInput opcode 1 - or possibly
  `XIQueryVersion` if this is XI2 - confirm from a live trace which
  one Xt actually sends before writing a parser).
- Same discipline as everywhere else in `extensions.md`: register
  XInputExtension in `KNOWN_EXTENSIONS`, do not flip
  `IMPLEMENTED_EXTENSIONS`/`present=1` until there's a real handler,
  implement exactly the one request a live trace shows Xt sending,
  verify live, stop.
- Do not guess at Xlib/XInput internals via general knowledge or web
  search as a substitute for checking the live trace or the actual
  Cygwin binaries - that produced a wrong hypothesis earlier in this
  investigation (assumed the `QueryExtension` call itself was fatal;
  the trace disproved this).

## Verify live

Per `.agents/memories/server.md`: `scripts/run_server.ps1` +
`scripts/run_xeyes.ps1`, `RUST_LOG=trace`, read
`scripts/server_trace.log` and `scripts/xeyes.log`. Success looks like
xeyes staying up (script doesn't throw "exited immediately") instead
of the current always-reproducing disconnect.
