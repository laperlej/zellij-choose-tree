use zellij_tile::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::sessiontree::Node;

pub struct Tab {
    name: String,
    position: usize,
    session: usize,
    session_name: String,
    shown: bool,
}

impl Tab {
    pub fn new(name: String, position: usize, session: usize, session_name: String) -> Self {
        Self {
            name,
            position,
            session,
            session_name,
            shown: false,
        }
    }
}

impl Node for Tab {
    fn id(&self) -> usize {
        self.position
    }
    fn identifier(&self) -> String {
        self.name.clone()
    }
    fn focus(&self) -> Result<(), String> {
        switch_session_with_focus(&self.session_name, Some(self.position), None);
        hide_self();
        Ok(())
    }
    fn kill(&self) -> Result<(), String> {
        Err("cannot kill tab".to_string())
    }
    fn session(&self) -> usize {
        self.session
    }
    fn tabs(&self) -> Rc<RefCell<Vec<usize>>> {
        Rc::new(RefCell::new(vec![]))
    }
    fn is_shown(&self) -> bool {
        self.shown
    }
    fn show(&mut self) {
        self.shown = true;
    }
    fn hide(&mut self) {
        self.shown = false
    }
    fn is_expanded(&self) -> bool {
        false
    }
    fn expand(&mut self) {
    }
    fn collapse(&mut self) {
    }
    fn render(&self, keybind: String, is_selected: bool) -> NestedListItem {
        let text = format!("({0}) {1}", keybind, self.name);
        let text_len = text.len();
        match is_selected {
            true => NestedListItem::new(text).indent(1).color_range(0, 0..text_len).selected(),
            false => NestedListItem::new(text).indent(1),
        }
    }
}
