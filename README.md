# zellij-choose-tree

[showcase.webm](https://github.com/user-attachments/assets/8a8e19d9-f527-4dfe-9952-a1aec1ba7ef0)

zellij-choose-tree is a plugin for [zellij](https://github.com/zellij-org/zellij) that allows users to quickly switch between sessions.

It aims to mimic the choose tree in [tmux](https://github.com/tmux/tmux/) accessible with `Ctrl+b s` by default.

## Usage

- Up/Down k/j arrow keys to navigate
- Left/Right h/l to fold/unfold sessions and reveal it's tabs
- `x` to delete selected session
- `Enter` to switch to selected session or tab
- `1-9` `A-Z` to switch to session or tab without navigating

## Installation

Download zellij-choose-tree.wasm from the [latest release](https://github.com/laperlej/zellij-choose-tree/releases/latest) and place it in your zellij plugins folder.

```bash
mkdir -p ~/.config/zellij/plugins
wget https://github.com/laperlej/zellij-choose-tree/releases/latest/download/zellij-choose-tree.wasm -O ~/.config/zellij/plugins/zellij-choose-tree.wasm
```

## Configuration

Add the plugin to a keybinding in your config.toml.

In this example, the keybinding is bound to `s` in tmux mode.

```kdl

tmux {
    # other keybinds here ...
    bind "s" { LaunchOrFocusPlugin "file:~/.config/zellij/plugins/zellij-choose-tree.wasm" {
            floating true
            move_to_focused_tab true
        }; SwitchToMode "Locked";
    }
}
```

## Contributing

Contributions are welcome. Please open an issue or a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
