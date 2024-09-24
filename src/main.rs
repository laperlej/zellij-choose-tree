use zellij_tile::prelude::*;
mod sessiontree;

use sessiontree::SessionTree;

use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    session_tree: SessionTree,

    message: String,
}

register_plugin!(State);


impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[PermissionType::ReadApplicationState, PermissionType::RunCommands, PermissionType::ChangeApplicationState]);
        subscribe(&[EventType::SessionUpdate, EventType::Key, EventType::Visible]);
    }

    fn update(&mut self, event: Event) -> bool {
        self.session_tree.update(event)
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
