# zellij-session-tree

zellij-session-tree is a plugin for [zellij](https://github.com/zellij-org/zellij) that displays a tree of sessions in the status bar.

It aims to mimic the session tree in [tmux](https://github.com/tmux/tmux/) accessible with `Ctrl+b s` by default.

## What functionalities are provided so far ?

- Tree navigation with arrow keys and vim directions
- Delete session
- Switch to session
- Switch to tab

## Usage

- Up/Down k/j arrow keys to navigate
- Left/Right h/l to fold/unfold sessions and reveal it's tabs
- `x` to delete selected session
- `Enter` to switch to selected session or tab
- `1-9` `A-Z` to switch to session or tab without navigating

## Installation

Download zellij-session-tree.wasm from the [latest release](https://github.com/zellij-org/zellij-session-tree/releases/latest) and place it in your zellij plugins folder.

## Contributing

Contributions are welcome. Please open an issue or a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
