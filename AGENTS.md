# AGENTS.md

Guidance for Claude Code and other coding agents working in this repository.

A zellij plugin: a session, tab and pane picker modelled on tmux `choose-tree`. It
compiles to WebAssembly and runs inside zellij, so it cannot be run or tested as an
ordinary binary.

## Build

```sh
cargo build            # debug
cargo build --release  # what the release workflow ships
```

`.cargo/config.toml` sets the target to `wasm32-wasip1`. Do not pass `--target` and do
not add a second target directory. Artifacts land in
`target/wasm32-wasip1/{debug,release}/zellij-choose-tree.wasm`.

Build with the rustup toolchain. A Homebrew or distro `rustc` earlier in `PATH` has no
wasm std, and the failure reads `can't find crate for 'core'`, which names neither the
toolchain nor `PATH`. Use `rustup run stable cargo build` if anything shadows rustup.

There are no unit tests, and adding one will not link: zellij's host functions are
undefined outside wasm, so `cargo check` passes for the host target but `cargo test`
fails at link time. Verification is a build plus the end-to-end check below.

## Architecture

`SessionTree` holds a flat `Vec` of nodes implementing `Node`, in display order: a
session, then its tabs, then each tab's panes. `cursor` indexes that vector, and
`quick_find` maps a displayed number shortcut to a node index.

`Node::identifier()` is stable for sessions (name) and panes (pane id). A tab reports
its **position**, so anything that must survive a tab being moved cannot key on a tab
identifier.

Anything derived from the node vector has to be rebuilt whenever the vector is, in the
same pass. `quick_find` was once appended to instead, which left the first render's
mapping in place and silently sent every number shortcut to the wrong node.

## Testing against a real zellij

The plugin only misbehaves in a live session. The method (throwaway session under a pty,
loading the plugin from a layout, asserting on cell attributes rather than text, teardown)
is published as a skill:
<https://github.com/laperlej/skills/tree/main/skills/zellij-plugin-testing>

What is specific to this plugin:

- `zellij action dump-screen <file>` strips every escape byte. The selected row is a
  background colour, so the cursor is invisible to it, and a test that moves the cursor
  and diffs the dumped text passes while proving nothing.
- Plugin keys (`j`, `k`, `l`, `h`, digits) go through `zellij action write-chars`. Zellij
  keybinds, such as a `LaunchOrFocusPlugin` binding, are consumed by the client before any
  pane sees them, so they must arrive as terminal input: `tui-test key press alt+o`.
- `zellij action launch-or-focus-plugin` hangs and never returns. Trigger a keybind.
- Configuration reaches the plugin only through a layout file, never a flag. The keys are
  `show_plugins` and `expanded` (`src/config.rs`), and each must be a KDL child node on
  its own line. Written inline, `{ expanded true }` fails to parse, and zellij leaves the
  session showing something else, usually the startup tip pane, so assertions run against
  the wrong surface. Wait for a known string from the plugin before asserting anything.
- On first load the plugin requests five permissions (`src/main.rs`) and the pane shows
  `Allow? (y/n)`. Answer `y`. It is asked once per plugin URL, so later runs skip it and a
  test that always expects the prompt will hang.

Every test that sends a keystroke needs a positive control: press something whose effect
is visible, and fail when nothing changes. Without one a test passes while the input
goes nowhere.

## Zellij plugin API

- `Event::Visible` never fires for this plugin. Nothing may depend on it.
- `Event::SessionUpdate` is event-driven, not periodic: nothing arrives while the
  session is idle, and a burst arrives when tabs or panes change.
- A plugin instance's identity includes its configuration. Two keybinds pointing at the
  same wasm with different options produce two independent instances.
- `LaunchOrFocusPlugin` focuses the tab holding the existing instance. Add
  `move_to_focused_tab true` to bring it to the current tab instead.

## Releasing

[`docs/RELEASING.md`](docs/RELEASING.md) is the source of truth: versioning, the commands
to cut a release, and the two ways to end up with a release that has no asset attached.
It is kept identical in zellij-sessionizer, so a change here belongs there too.

## Do not

- Commit `target/`.
- Hand-edit `Cargo.lock` beyond the version bump a release implies.
