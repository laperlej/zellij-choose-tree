use zellij_tile::prelude::*;
use crate::utils::{IdGenerator, KeybindGenerator};
use crate::session::Session;
use crate::tab::Tab;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default)]
pub struct SessionTree {
    nodes: Vec<Box::<dyn Node>>,
    cursor: i32,
    quick_find: Vec<usize>,
}

impl From<Vec<SessionInfo>> for SessionTree {
    fn from(sessions: Vec<SessionInfo>) -> Self {
        let mut nodes: Vec<Box<dyn Node>> = Vec::new();
        let mut id_generator = IdGenerator::new();
        for session in sessions.iter() {
            let session_id = id_generator.next();
            let tab_ids = Rc::new(RefCell::new(Vec::new()));
            let session_node = Session::new(session_id, session.name.clone(), session.is_current_session, &tab_ids);
            nodes.push(Box::new(session_node));
            for tab in session.tabs.iter() {
                let tab_id = id_generator.next();
                tab_ids.borrow_mut().push(tab_id);
                let tab_node = Tab::new(tab.name.clone(), tab.position, session_id, session.name.clone());
                nodes.push(Box::new(tab_node));
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
    fn id(&self) -> usize;
    fn focus(&self) -> Result<(), String>;
    fn kill(&self) -> Result<(), String>;
    fn session(&self) -> usize;
    fn tabs(&self) -> Rc<RefCell<Vec<usize>>>;
    fn is_shown(&self) -> bool;
    fn show(&mut self);
    fn hide(&mut self);
    fn is_expanded(&self) -> bool;
    fn expand(&mut self);
    fn collapse(&mut self);
    fn render(&self, keybind: String, is_selected: bool) -> NestedListItem;
}



impl SessionTree {
    pub fn get_current_node(&self) -> Result<&dyn Node, String> {
        let session = self.nodes.get(self.cursor as usize).ok_or("cursor out of range")?;
        Ok(session.as_ref())
    }

    pub fn get_session(&mut self, index: usize) -> Result<&mut Box<dyn Node>, String> {
        let session_index = self.get_node(index)?.session();
        let session = self.get_node(session_index)?;
        Ok(session)
    }

    pub fn get_node(&mut self, index: usize) -> Result<&mut Box<dyn Node>, String> {
        let session = self.nodes.get_mut(index).ok_or("session index out of range")?;
        Ok(session)
    }

    pub fn expand(&mut self, index: usize) -> Result<(), String> {
        let session = self.get_session(index)?;
        session.expand();
        for tab_index in session.tabs().borrow().iter() {
            let tab = self.get_node(*tab_index)?;
            tab.show();
        }
        Ok(())
    }
    pub fn collapse(&mut self, index: usize) -> Result<(), String> {
        let session = self.get_session(index)?;
        let next_position = session.id() as i32;
        session.collapse();
        for tab_index in session.tabs().borrow().iter() {
            let tab = self.get_node(*tab_index)?;
            tab.hide();
        }
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
            if self.get_current_node()?.is_shown() {
                break;
            }
        }
        Ok(())
    }

    pub fn handle_up(&mut self) -> Result<(), String> {
        for _ in 0..=self.nodes.len() {
            self.wraping_previous();
            if self.get_current_node()?.is_shown() {
                break;
            }
        }
        Ok(())
    }

    pub fn handle_left(&mut self) -> Result<(), String> {
        let session = self.get_session(self.cursor as usize)?;
        if session.is_expanded() {
            self.collapse(self.cursor as usize)?;
        } else {
            for _ in 0..=self.nodes.len() {
                self.saturating_previous();
                if self.get_current_node()?.is_shown() {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn handle_right(&mut self) -> Result<(), String> {
        let session = self.get_session(self.cursor as usize)?;
        if !session.is_expanded() {
            self.expand(self.cursor as usize)?;
        } else {
            for _ in 0..=self.nodes.len() {
                self.saturating_next();
                if self.get_current_node()?.is_shown() {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn switch_by_index(&mut self, target: usize) -> Result<(), String> {
        let node_id = self.quick_find.get(target).ok_or("quick_find index out of range")?;
        let node = self.get_node(*node_id)?;
        node.focus()?;
        Ok(())
    }

    pub fn switch_to_selected(&self) -> Result<(), String> {
        let node = self.get_current_node()?;
        node.focus()?;
        Ok(())
    }

    pub fn kill_selected(& self) -> Result<(), String> {
        let node = self.get_current_node()?;
        node.kill()?;
        Ok(())
    }

    pub fn render(&mut self, rows: usize, _cols: usize) {
        let mut keybind_generator = KeybindGenerator::new();
        let mut lines = Vec::new();
        for (i, node) in self.nodes.iter().enumerate().filter(|(_, node)| node.is_shown()) {
            let text = node.render(keybind_generator.next(), i == self.cursor as usize);
            lines.push(text);
            self.quick_find.push(i);
        }
        let from = (self.cursor as usize).saturating_sub(rows.saturating_sub(1) / 2).min(lines.len().saturating_sub(rows));
        print_nested_list(lines.into_iter().skip(from).take(rows).collect());
    }
}
