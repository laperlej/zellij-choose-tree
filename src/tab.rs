use zellij_tile::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::sessiontree::Node;

pub struct Tab {
    index: usize,
    name: String,
    position: usize,
    active: bool,
    session: Rc<RefCell<dyn Node>>,
    panes: Vec<Rc<RefCell<dyn Node>>>,
    shown: bool,
    is_expanded: bool,
}

impl Tab {
    pub fn new(index: usize, name: String, position: usize, session: Rc<RefCell<dyn Node>>, active: bool) -> Self {
        Self {
            index,
            name,
            position,
            active,
            session: session.clone(),
            panes: Vec::new(),
            shown: false,
            is_expanded: false,
        }
    }
}

impl Node for Tab {
    fn index(&self) -> usize {
        self.index
    }
    fn identifier(&self) -> String {
        self.position.to_string()
    }
    fn is_focused(&self) -> bool {
        self.active
    }
    fn focus(&self) -> Result<(), String> {
        let session = self.session.borrow();
        if session.is_focused() {
            focus_or_create_tab(self.name.as_str());
        } else {
            switch_session_with_focus(&session.identifier(), Some(self.position), None);
        }
        hide_self();
        Ok(())
    }
    fn kill(&self) -> Result<(), String> {
        Err("cannot kill tab".to_string())
    }
    fn parent(&self) -> Option<Rc<RefCell<dyn Node>>> {
        Some(self.session.clone())
    }
    fn add_child(&mut self, child: Rc<RefCell<dyn Node>>) {
        self.panes.push(child);
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
         self.is_expanded
    }
    fn expand(&mut self) {
        if self.is_expanded {
            return;
        }
        self.is_expanded = true;
        for pane in self.panes.iter() {
            pane.borrow_mut().show();
        }
    }
    fn collapse(&mut self) {
        if !self.is_expanded {
            return;
        }
        self.is_expanded = false;
        for pane in self.panes.iter() {
            pane.borrow_mut().hide();
        }
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
