use zellij_tile::prelude::*;

use std::collections::{HashSet, BTreeMap};

#[derive(Default)]
struct State {
    sessions: Vec<SessionInfo>,
    nodes: Vec<Node>,
    cursor: usize,
    expanded: HashSet<String>,
    userspace_configuration: BTreeMap<String, String>,
    message: String,
}

register_plugin!(State);

#[derive(Debug, Clone)]
enum Node {
    Session(SessionInfo),
    Tab(TabInfo),
}

impl Node {
    fn is_session(&self) -> bool {
        match self {
            Node::Session(_) => true,
            Node::Tab(_) => false,
        }
    }
}

impl State {
    fn refresh_list(&mut self) {
        let mut new_nodes: Vec<Node> = Vec::new();
        for session in self.sessions.iter() {
            new_nodes.push(Node::Session(session.clone()));
            if self.is_expanded(&session.name) {
                for tab in session.tabs.iter() {
                    new_nodes.push(Node::Tab(tab.clone()));
                }
            }
        }
        self.nodes = new_nodes;
    }

    fn parent(&self, mut i: usize) -> usize {
        while i > 0 && !self.nodes[i].is_session() {
            i -= 1;
        }
        i
    }

    fn is_expanded(&self, session_name: &str) -> bool {
        self.expanded.contains(session_name)
    }

    fn expand(&mut self, session_name: &str) {
        self.expanded.insert(session_name.to_string());
        self.refresh_list();
    }

    fn collapse(&mut self, session_name: &str) {
        self.expanded.remove(session_name);
        self.refresh_list();
    }

    fn move_down(&mut self) {
        if self.cursor < self.nodes.len().saturating_sub(1) {
            self.cursor += 1;
        } else {
            self.cursor = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        } else {
            self.cursor = self.nodes.len().saturating_sub(1);
        }
    }
    
    fn goto(&mut self, node: &Node) {
        match node {
            Node::Session(session) => {
                if !session.is_current_session {
                    let session_name = session.name.clone();
                    switch_session_with_focus(&session_name, Some(0), None);
                }
            },
            Node::Tab(tab) => {
                let session_idx = self.parent(self.cursor);
                let session = match &self.nodes[session_idx] {
                    Node::Session(session) => session,
                    _ => return
                };
                if !session.is_current_session {
                    let session_name = session.name.clone();
                    let tab_position = tab.position;
                    switch_session_with_focus(&session_name, Some(tab_position), None);
                }
            }
        };
        hide_self();
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.userspace_configuration = configuration;
        // we need the ReadApplicationState permission to receive the ModeUpdate and TabUpdate
        // events
        // we need the RunCommands permission to run "cargo test" in a floating window
        request_permission(&[PermissionType::ReadApplicationState, PermissionType::RunCommands, PermissionType::ChangeApplicationState]);
        subscribe(&[EventType::SessionUpdate, EventType::Key, EventType::Visible]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event { 
            Event::SessionUpdate(sessions, _) => {
                self.sessions = sessions;
                self.refresh_list();
                should_render = true;
            }
            Event::Key(key) => {
                match key {
                    Key::Char('\n') => {
                        let current_node = self.nodes[self.cursor].clone();
                        self.goto(&current_node);
                    }
                    Key::Char('0') | Key::Char('1') | Key::Char('2') | Key::Char('3') | Key::Char('4') | Key::Char('5') | Key::Char('6') | Key::Char('7') | Key::Char('8') | Key::Char('9') => {
                        let index = key.to_string().parse::<usize>().unwrap();
                        if index < self.nodes.len() {
                            let node = self.nodes[index].clone();
                            self.goto(&node);
                        }
                    },
                    Key::Char('k') | Key::Up => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        } else {
                            self.cursor = self.nodes.len().saturating_sub(1);
                        }
                        should_render = true;
                    }
                    Key::Char('j') | Key::Down => {
                        if self.cursor < self.nodes.len().saturating_sub(1) {
                            self.cursor += 1;
                        } else {
                            self.cursor = 0;
                        }
                        should_render = true;
                    }
                    Key::Char('h') | Key::Left => {
                        let current_node = &self.nodes[self.cursor];
                        match current_node {
                            Node::Session(session) => {
                                if self.is_expanded(&session.name) {
                                    self.collapse(&session.name.clone());
                                } else if self.cursor > 0 {
                                    self.move_up();
                                }
                            },
                            Node::Tab(_tab) => {
                                let session_idx = self.parent(self.cursor);
                                let session = match &self.nodes[session_idx] {
                                    Node::Session(session) => session,
                                    _ => return false,
                                };
                                self.cursor = session_idx;
                                self.collapse(&session.name.clone());
                            }
                        };
                        should_render = true;
                    }
                    Key::Char('l') | Key::Right => {
                        let current_node = &self.nodes[self.cursor];
                        match current_node {
                            Node::Session(session) => {
                                if !self.is_expanded(&session.name.clone()) {
                                    self.expand(&session.name.clone());
                                } else if self.cursor < self.nodes.len().saturating_sub(1) {
                                    self.move_down();
                                }
                            },
                            Node::Tab(_tab) => {
                                if self.cursor < self.nodes.len().saturating_sub(1) {
                                    self.move_down();
                                }
                            }
                        };
                        should_render = true;
                    }
                    Key::Char('x') | Key::Delete => {
                        let current_node = &self.nodes[self.cursor];
                        match current_node {
                            Node::Session(session) => {
                                kill_sessions(&[session.name.clone()]);
                            },
                            Node::Tab(_tab) => {
                            }
                        };
                        should_render = true;
                    }
                    Key::Esc => {
                        self.cursor = 0;
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
        let nested_list = self.nodes.iter().enumerate().map(|(i, node)| {
            let item = match node {
                Node::Session(session) => {
                    let is_current = session.is_current_session;
                    let text = match is_current {
                        true => format!("({0}) {1} (attached)", i, session.name),
                        false => format!("({0}) {1}", i, session.name),
                    };
                    NestedListItem::new(text)
                },
                Node::Tab(tab) => {
                    let is_current = tab.active;
                    let text = match is_current {
                        true => format!("({0}) {1} (active)", i, tab.name),
                        false => format!("({0}) {1}", i, tab.name),
                    };
                    NestedListItem::new(text).indent(1)
                }
            };
            if i == self.cursor {
                item.selected()
            } else {
                item
            }
        }).collect();
        print_nested_list(nested_list);
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
