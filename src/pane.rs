use zellij_tile::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::sessiontree::Node;

pub struct Pane {
    index: usize,
    title: String,
    is_focused: bool,
    pane_id: (u32, bool), //(id, is_plugin)
    tab: Rc<RefCell<dyn Node>>,
    shown: bool,
}

impl Pane {
    pub fn new(index: usize, title: String, pane_id: (u32, bool), tab: Rc<RefCell<dyn Node>>, is_focused: bool) -> Self {
        Self {
            index,
            title,
            is_focused,
            pane_id,
            tab: tab.clone(),
            shown: false,
        }
    }
}

impl Node for Pane {
    fn index(&self) -> usize {
        self.index
    }
    fn identifier(&self) -> String {
        self.pane_id.0.to_string()
    }
    fn is_focused(&self) -> bool {
        self.is_focused
    }
    fn focus(&self) -> Result<(), String> {
        let tab = self.tab.borrow();
        let tab_position = tab.identifier().parse().map_err(|_| "tab identifier is not a number")?;
        let session = tab.parent().ok_or("tab has no parent")?;
        let session_name = session.borrow().identifier();
        if session.borrow().is_focused() {
            focus_terminal_pane(self.pane_id.0, true);
        } else {
            switch_session_with_focus(&session_name, Some(tab_position), Some(self.pane_id));
        }
        hide_self();
        Ok(())
    }
    fn kill(&self) -> Result<(), String> {
        Err("cannot kill pane".to_string())
    }
    fn parent(&self) -> Option<Rc<RefCell<dyn Node>>> {
        Some(self.tab.clone())
    }
    fn add_child(&mut self, _child: Rc<RefCell<dyn Node>>) {
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
        true
    }
    fn expand(&mut self) {
    }
    fn collapse(&mut self) {
    }
    fn render(&self, keybind: String, is_selected: bool) -> NestedListItem {
        let text = format!("({0}) {1}", keybind, self.title);
        let text_len = text.len();
        match is_selected {
            true => NestedListItem::new(text).indent(2).color_range(0, 0..text_len).selected(),
            false => NestedListItem::new(text).indent(2),
        }
    }
}
