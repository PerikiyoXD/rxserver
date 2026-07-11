# Task: fix the post-setup client disconnect at InternAtom(WM_PROTOCOLS)

## What's confirmed (live trace)

The XInputExtension disconnect this repo was chasing for several
sessions is resolved (see `.agents/memories/extensions.md` for what's
implemented: XInputExtension's `GetExtensionVersion`, `XIQueryVersion`,
`XISelectEvents`, and XKEYBOARD's `UseExtension`). xeyes/Xt now
completes the entire XInput/XKB negotiation chain without error and
proceeds into normal window setup exactly like a baseline run with no
XInputExtension traffic at all.

A different, older disconnect remains, present even before any
XInputExtension work started this session: the client finishes a
batch of requests ending in `InternAtom` for `WM_PROTOCOLS`, then
closes its TCP socket before the server's reply write for that last
request completes. The server logs this as:

```text
ERROR ... rxserver::server::connection: Request processing failed for client 1:
Failed to send response
```

which is `connection.rs`'s `write_all(...).context("Failed to send
response")` failing - a real socket error, not a bug in that write
call itself. `scripts/xeyes.log` for this failure mode is empty or
just shows `X connection to 127.0.0.1:0 broken (explicit kill or
server shutdown).` with no Xlib fatal-extension message (that message
was specific to the now-resolved XInputExtension issue).

## What's not yet done

- Determine why the client closes its socket right after sending
  `InternAtom(WM_PROTOCOLS)` instead of waiting for the reply. Xt's
  own request pipelining means it may have already decided to
  disconnect for an unrelated reason (a subsequent Xlib/Xt call that
  fails fatally, same "deferred fatal" pattern the XInputExtension
  investigation uncovered) rather than this specific request being
  the trigger.
- Check whether xeyes fully renders/functions before this disconnect,
  or if the disconnect happens before the window becomes interactive
  - this determines whether it's cosmetic (clean shutdown once the
    UI has already appeared) or a real functional blocker.
- Same discipline as always: verify live via `scripts/run_server.ps1`
  + `scripts/run_xeyes.ps1`, read `scripts/server_trace.log` and
  `scripts/xeyes.log`, decode raw request bytes rather than assume
  wire formats from memory, implement exactly the next confirmed gap,
  stop, report.

## Verify live

Per `.agents/memories/server.md`. Success looks like xeyes staying up
indefinitely (script doesn't throw "exited immediately") with the eyes
visibly tracking the pointer, not just the connection surviving one
batch further than before.
