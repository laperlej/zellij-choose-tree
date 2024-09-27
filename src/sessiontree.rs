use zellij_tile::prelude::*;
use crate::utils::{IdGenerator, KeybindGenerator};
use crate::session::Session;
use crate::tab::Tab;
use crate::pane::Pane;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default)]
pub struct SessionTree {
    nodes: Vec<Rc<RefCell<dyn Node>>>,
    cursor: i32,
    quick_find: Vec<usize>,
}

impl From<Vec<SessionInfo>> for SessionTree {
    fn from(sessions: Vec<SessionInfo>) -> Self {
        let mut nodes: Vec<Rc<RefCell<dyn Node>>> = Vec::new();
        let mut id_generator = IdGenerator::new();
        for session in sessions.iter() {
            let session_index = id_generator.next();
            let session_node: Rc<RefCell<dyn Node>> = Rc::new(RefCell::new(Session::new(session_index, session.name.clone(), session.is_current_session)));
            nodes.push(session_node.clone());
            for tab in session.tabs.iter() {
                let tab_index = id_generator.next();
                let tab_node = Rc::new(RefCell::new(Tab::new(tab_index, tab.name.clone(), tab.position, session_node.clone())));
                session_node.borrow_mut().add_child(tab_node.clone());
                nodes.push(tab_node.clone());
                for pane in session.panes.panes.get(&tab.position).unwrap_or(&Vec::new()) {
                    let pane_index = id_generator.next();
                    let pane_node = Rc::new(RefCell::new(Pane::new(pane_index, pane.title.clone(), (pane.id, pane.is_plugin), tab_node.clone())));
                    tab_node.borrow_mut().add_child(pane_node.clone());
                    nodes.push(pane_node.clone());
                }
            }
        }
        Self {
            nodes,
            cursor: 0,
            quick_find: Vec::new(),
        }
    }
}

pub trait Node {
    fn index(&self) -> usize;
    fn identifier(&self) -> String;
    fn focus(&self) -> Result<(), String>;
    fn kill(&self) -> Result<(), String>;
    fn parent(&self) -> Option<Rc<RefCell<dyn Node>>>;
    fn add_child(&mut self, child: Rc<RefCell<dyn Node>>);
    fn is_shown(&self) -> bool;
    fn show(&mut self);
    fn hide(&mut self);
    fn is_expanded(&self) -> bool;
    fn expand(&mut self);
    fn collapse(&mut self);
    fn render(&self, keybind: String, is_selected: bool) -> NestedListItem;
}



impl SessionTree {
    pub fn get_current_node(&self) -> Result<Rc<RefCell<dyn Node>>, String> {
        let node = self.nodes.get(self.cursor as usize).ok_or("cursor out of range")?;
        Ok(node.clone())
    }

    pub fn get_parent(&self, index: usize) -> Result<Rc<RefCell<dyn Node>>, String> {
        let session = match self.get_node(index)?.borrow().parent() {
            Some(parent) => parent,
            None => self.get_node(index)?,
        };
        Ok(session)
    }

    pub fn get_session(&self, index: usize) -> Result<Rc<RefCell<dyn Node>>, String> {
        let mut current = self.get_node(index)?.clone();
        while current.borrow().parent().is_some() {
            current = self.get_parent(current.clone().borrow().index())?;
        }
        Ok(current)
    }

    pub fn get_node(&self, index: usize) -> Result<Rc<RefCell<dyn Node>>, String> {
        let session = self.nodes.get(index).ok_or("session index out of range")?;
        Ok(session.clone())
    }

    pub fn expand(&mut self, index: usize) -> Result<(), String> {
        let node = self.get_node(index)?;
        node.borrow_mut().expand();
        Ok(())
    }
    pub fn collapse(&mut self, index: usize) -> Result<(), String> {
        let parent = self.get_parent(index)?;
        let next_position = parent.borrow().index() as i32;
        parent.borrow_mut().collapse();
        self.cursor = next_position;
        Ok(())
    }

    fn wraping_previous(&mut self) {
        if self.cursor == 0 {
            self.cursor = (self.nodes.len() - 1) as i32;
        } else {
            self.cursor -= 1;
        }
    }

    fn wraping_next(&mut self) {
        if self.cursor as usize == self.nodes.len() - 1 {
            self.cursor = 0;
        } else {
            self.cursor += 1;
        }
    }

    fn saturating_previous(&mut self) {
        self.cursor = 0.max(self.cursor - 1);
    }

    fn saturating_next(&mut self) {
        self.cursor = (self.nodes.len() as i32 - 1).min(self.cursor + 1)
    }


    pub fn handle_down(&mut self) -> Result<(), String> {
        for _ in 0..=self.nodes.len() {
            self.wraping_next();
            if self.get_current_node()?.borrow().is_shown() {
                break;
            }
        }
        Ok(())
    }

    pub fn handle_up(&mut self) -> Result<(), String> {
        for _ in 0..=self.nodes.len() {
            self.wraping_previous();
            if self.get_current_node()?.borrow().is_shown() {
                break;
            }
        }
        Ok(())
    }

    pub fn handle_left(&mut self) -> Result<(), String> {
        let parent = self.get_parent(self.cursor as usize)?;
        if parent.borrow().is_expanded() {
            self.collapse(self.cursor as usize)?;
        } else {
            for _ in 0..=self.nodes.len() {
                self.saturating_previous();
                if self.get_current_node()?.borrow().is_shown() {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> Result<(), String> {
        let current = self.get_current_node()?;
        if !current.borrow().is_expanded() {
            self.expand(self.cursor as usize)?;
        } else {
            for _ in 0..=self.nodes.len() {
                self.saturating_next();
                if self.get_current_node()?.borrow().is_shown() {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn switch_by_index(&mut self, target: usize) -> Result<(), String> {
        let node_id = self.quick_find.get(target).ok_or("quick_find index out of range")?;
        let node = self.get_node(*node_id)?;
        node.borrow().focus()?;
        Ok(())
    }

    pub fn switch_to_selected(&self) -> Result<(), String> {
        let node = self.get_current_node()?;
        node.borrow().focus()?;
        Ok(())
    }

    pub fn kill_selected(& self) -> Result<(), String> {
        let node = self.get_current_node()?;
        node.borrow().kill()?;
        Ok(())
    }

    pub fn render(&mut self, rows: usize, _cols: usize) {
        let mut keybind_generator = KeybindGenerator::new();
        let mut lines = Vec::new();
        for (i, node) in self.nodes.iter().enumerate().filter(|(_, node)| node.borrow().is_shown()) {
            let text = node.borrow().render(keybind_generator.next(), i == self.cursor as usize);
            lines.push(text);
            self.quick_find.push(i);
        }
        let from = (self.cursor as usize).saturating_sub(rows.saturating_sub(1) / 2).min(lines.len().saturating_sub(rows));
        print_nested_list(lines.into_iter().skip(from).take(rows).collect());
    }
}
