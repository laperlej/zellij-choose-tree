use zellij_tile::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::sessiontree::Node;

pub struct Session {
    index: usize,
    name: String,
    is_current_session: bool,
    tabs: Vec<Rc<RefCell<dyn Node>>>,
    is_expanded: bool,
}

impl Session {
    pub fn new(index: usize, name: String, is_current_session: bool) -> Self {
        Self {
            index,
            name,
            is_current_session,
            tabs: Vec::new(),
            is_expanded: false,
        }
    }
}

impl Node for Session {
    fn index(&self) -> usize {
        self.index
    }
    fn identifier(&self) -> String {
        self.name.clone()
    }
    fn focus(&self) -> Result<(), String> {
        if self.is_current_session {
            hide_self();
            return Err("cannot goto current session".to_string());
        }
        switch_session(Some(&self.name));
        hide_self();
        Ok(())
    }
    fn kill(&self) -> Result<(), String> {
        kill_sessions(&[self.name.clone()]);
        Ok(())
    }
    fn parent(&self) -> Option<Rc<RefCell<dyn Node>>> {
        None
    }
    fn add_child(&mut self, child: Rc<RefCell<dyn Node>>) {
        self.tabs.push(child);
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
        if self.is_expanded {
            return;
        }
        self.is_expanded = true;
        for tab in self.tabs.iter() {
            tab.borrow_mut().show();
        }
    }
    fn collapse(&mut self) {
        if !self.is_expanded {
            return;
        }
        self.is_expanded = false;
        for tab in self.tabs.iter() {
            tab.borrow_mut().collapse();
            tab.borrow_mut().hide();
        }
    }
    fn render(&self, keybind: String, is_selected: bool) -> NestedListItem {
        let text = match self.is_current_session {
            true => format!("({0}) {1} (attached)", keybind, self.name),
            false => format!("({0}) {1}", keybind, self.name),
        };
        let text_len = text.len();
        match is_selected {
            true => NestedListItem::new(text).indent(0).color_range(0, 0..text_len).selected(),
            false => NestedListItem::new(text).indent(0),
        }
    }
}
