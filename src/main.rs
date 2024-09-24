use zellij_tile::prelude::*;
mod sessiontree;

use sessiontree::SessionTree;

use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    session_tree: SessionTree,
    initialised: bool,

    message: String,
}

register_plugin!(State);


impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[PermissionType::ReadApplicationState, PermissionType::RunCommands, PermissionType::ChangeApplicationState]);
        subscribe(&[EventType::SessionUpdate, EventType::Key, EventType::Visible]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event { 
            Event::SessionUpdate(sessions, _) => {
                if !self.initialised {
                    self.session_tree = SessionTree::from(sessions);
                    self.initialised = true;
                    should_render = true;
                }
            }
            Event::Key(key) => {
                match key {
                    // Select the node under the cursor
                    Key::Char('\n') => {
                        self.session_tree.switch_to_selected();
                        hide_self();
                    }
                    // Select the node at the given index
                    Key::Char(c) if c.is_ascii_digit() => {
                        if let Some(digit) = c.to_digit(10) {
                            self.session_tree.switch_by_index(digit as usize);
                        }
                    },
                    // Move up, looping around
                    Key::Char('k') | Key::Up => {
                        self.session_tree.handle_up();
                        should_render = true;
                    }
                    // Move down, looping around
                    Key::Char('j') | Key::Down => {
                        self.session_tree.handle_down();
                        should_render = true;
                    }
                    // Collapse the current node, moving up if already collapsed
                    Key::Char('h') | Key::Left => {
                        self.session_tree.handle_left();
                        should_render = true;
                    }
                    // Expand the current node, moving down if already expanded
                    Key::Char('l') | Key::Right => {
                        self.session_tree.handle_right();
                        should_render = true;
                    }
                    // Kill the current node
                    Key::Char('x') | Key::Delete => {
                        self.session_tree.kill_selected();
                        should_render = true;
                    }
                    // Quit
                    Key::Esc => {
                        hide_self();
                    }
                    _ => (),
                }
            }
            _ => (),
        };
        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!();
        if !self.message.is_empty() {
            println!("{}", self.message);
            println!();
        }
        self.session_tree.render(_rows, _cols);
    }
}

pub const CYAN: u8 = 51;
pub const GRAY_LIGHT: u8 = 238;
pub const GRAY_DARK: u8 = 245;
pub const WHITE: u8 = 15;
pub const BLACK: u8 = 16;
pub const RED: u8 = 124;
pub const GREEN: u8 = 154;
pub const ORANGE: u8 = 166;
