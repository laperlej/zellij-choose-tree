use zellij_tile::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::sessiontree::Node;

pub struct Session {
    id: usize,
    name: String,
    is_current_session: bool,
    tabs: Rc<RefCell<Vec<usize>>>,
    is_expanded: bool,
}

impl Session {
    pub fn new(id: usize, name: String, is_current_session: bool, tabs: &Rc<RefCell<Vec<usize>>>) -> Self {
        Self {
            id,
            name,
            is_current_session,
            tabs: tabs.clone(),
            is_expanded: false,
        }
    }
}

impl Node for Session {
    fn id(&self) -> usize {
        self.id
    }
    fn focus(&self) -> Result<(), String> {
        if self.is_current_session {
            return Err("cannot goto current session".to_string());
        }
        switch_session(Some(&self.name));
        Ok(())
    }
    fn kill(&self) -> Result<(), String> {
        kill_sessions(&[self.name.clone()]);
        Ok(())
    }
    fn session(&self) -> usize {
        self.id
    }
    fn tabs(&self) -> Rc<RefCell<Vec<usize>>> {
        self.tabs.clone()
    }
    fn is_shown(&self) -> bool {
        true
    }
    fn show(&mut self) {
    }
    fn hide(&mut self) {
    }
    fn is_expanded(&self) -> bool {
        self.is_expanded
    }
    fn expand(&mut self) {
        self.is_expanded = true;
    }
    fn collapse(&mut self) {
        self.is_expanded = false;
    }
    fn render(&self, keybind: String, is_selected: bool) -> NestedListItem {
        let text = match self.is_current_session {
            true => format!("({0}) {1} (attached)", keybind, self.name),
            false => format!("({0}) {1}", keybind, self.name),
        };
        match is_selected {
            true => NestedListItem::new(text).indent(0).selected(),
            false => NestedListItem::new(text).indent(0),
        }
    }
}
