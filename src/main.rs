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

    fn expand(&mut self, session_name: String) {
        self.expanded.insert(session_name);
        self.refresh_list();
    }

    fn collapse(&mut self, session_name: String) {
        self.expanded.remove(&session_name);
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
    
    fn goto(&self, node: &Node) {
        match node {
            Node::Session(session) => {
                if !session.is_current_session {
                    let session_name = session.name.clone();
                    switch_session_with_focus(&session_name, Some(0), None);
                }
            },
            Node::Tab(tab) => {
                let session_idx = self.parent(self.cursor);
                let session = match &self.nodes.get(session_idx) {
                    Some(Node::Session(session)) => session,
                    _ => return,
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
                    // Select the node under the cursor
                    Key::Char('\n') => {
                        if let Some(node) = self.nodes.get(self.cursor) {
                            self.goto(node)
                        };
                    }
                    // Select the node at the given index
                    Key::Char(c) if c.is_ascii_digit() => {
                        if let Ok(index) = key.to_string().parse::<usize>() {
                            if let Some(node) = self.nodes.get(index) {
                                self.goto(node)
                            };
                        }
                    },
                    // Move up, looping around
                    Key::Char('k') | Key::Up => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        } else {
                            self.cursor = self.nodes.len().saturating_sub(1);
                        }
                        should_render = true;
                    }
                    // Move down, looping around
                    Key::Char('j') | Key::Down => {
                        if self.cursor < self.nodes.len().saturating_sub(1) {
                            self.cursor += 1;
                        } else {
                            self.cursor = 0;
                        }
                        should_render = true;
                    }
                    // Collapse the current node, moving up if already collapsed
                    Key::Char('h') | Key::Left => {
                        let current_node = &self.nodes[self.cursor];
                        match current_node {
                            Node::Session(session) => {
                                if self.is_expanded(&session.name) {
                                    self.collapse(session.name.clone());
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
                                self.collapse(session.name.clone());
                            }
                        };
                        should_render = true;
                    }
                    // Expand the current node, moving down if already expanded
                    Key::Char('l') | Key::Right => {
                        let current_node = &self.nodes[self.cursor];
                        match current_node {
                            Node::Session(session) => {
                                if !self.is_expanded(&session.name.clone()) {
                                    self.expand(session.name.clone());
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
                    // Kill the current node
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
                    // Quit
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
