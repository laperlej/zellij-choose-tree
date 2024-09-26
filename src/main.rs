mod sessiontree;
mod session;
mod tab;
mod utils;

use zellij_tile::prelude::*;
use std::collections::BTreeMap;

use sessiontree::SessionTree;

#[derive(Default)]
struct State {
    session_tree: SessionTree,
    initialised: bool,

    debug: String,
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
                        let _ = self.session_tree.switch_to_selected();
                        hide_self();
                    }
                    // Move up, looping around
                    Key::Char('k') | Key::Up => {
                        let _ = self.session_tree.handle_up();
                        should_render = true;
                    }
                    // Move down, looping around
                    Key::Char('j') | Key::Down => {
                        let _ = self.session_tree.handle_down();
                        should_render = true;
                    }
                    // Collapse the current node, moving up if already collapsed
                    Key::Char('h') | Key::Left => {
                        let _ = self.session_tree.handle_left();
                        should_render = true;
                    }
                    // Expand the current node, moving down if already expanded
                    Key::Char('l') | Key::Right => {
                        let _ = self.session_tree.handle_right();
                        should_render = true;
                    }
                    // Kill the current node
                    Key::Char('x') | Key::Delete => {
                        let _ = self.session_tree.kill_selected();
                        self.initialised = false;
                        should_render = true;
                    }
                    // Select the node at the given index
                    Key::Char(c) => {
                        if let Some(digit) = c.to_digit(10) {
                            let _ = self.session_tree.switch_by_index(digit as usize);
                        }
                        // Capital letters are used to select the node at the given index
                        else if c.is_ascii_uppercase() {
                            let index = 10 + c as u8 - b'A';
                            let _ = self.session_tree.switch_by_index(index as usize);
                        }
                        //hide_self();
                        should_render = true;
                    },
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

    fn render(&mut self, rows: usize, cols: usize) {
        println!();
        if !self.debug.is_empty() {
            println!("{}", self.debug);
            println!();
        }
        self.session_tree.render(rows.saturating_sub(3), cols);
    }
}
