# cefsrc crash repro

Crash happens when you have two pipelines with cefsrcs:

- start pipeline #1
- stop #1 either manually or via the new js-signals feature
- start #2
- observe segfault in CefInitialize call

This happens both on master (57f3ec1) and on the sha prior to the merge
of the signals fork (002f660) - where we just use sleeps to emulate the behaviour
of the signals.

The repo has two binaries:

- `cargo run --bin signals` repros the issue using the signals (this will only
  work on 57f3ec1)
- `cargo run --bin sleep` repros the issue on both master and 002f660
