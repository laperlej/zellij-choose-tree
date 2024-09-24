use zellij_tile::prelude::*;

#[derive(Default)]
pub struct SessionTree {
    cursor: Cursor,
    expanded: Vec<bool>,
    sessions: Vec<SessionInfo>,
}

#[derive(Debug, Default)]
struct Cursor {
    session: usize,
    tab: Option<usize>,
}

impl SessionTree {
    pub fn toggle(&mut self, state: bool) {
        if let Some(is_expanded) = self.expanded.get_mut(self.cursor.session) {
            *is_expanded = state;
        }
    }

    fn previous_with_cycle(&mut self) {
        if let Some(tab_idx) = self.cursor.tab {
            if tab_idx > 0 {
                self.cursor.tab = Some(tab_idx - 1);
                return;
            }
        }
        let next_session_idx = match self.cursor.session {
            0 => self.sessions.len().saturating_sub(1),
            _ => self.cursor.session.saturating_sub(1),
        };
        let next_tab_idx = match self.expanded.get(next_session_idx) {
            Some(true) => self.sessions.get(next_session_idx).map(|session| session.tabs.len().saturating_sub(1)),
            Some(false) => None,
            None => None,
        };
        self.cursor.session = next_session_idx;
        self.cursor.tab = next_tab_idx;
    }

    fn next_with_cycle(&mut self) {
        if let Some(session) = self.sessions.get(self.cursor.session) {
            if let Some(tab_idx) = self.cursor.tab {
                if tab_idx + 1 < session.tabs.len() {
                    self.cursor.tab = Some(tab_idx + 1);
                    return;
                }
            }
            let next_session_idx = self.cursor.session.wrapping_sub(1).min(self.sessions.len().saturating_sub(1));
            let next_tab_idx = match self.expanded.get(next_session_idx) {
                Some(true) => Some(0),
                Some(false) => None,
                None => None,
            };
            self.cursor.session = next_session_idx;
            self.cursor.tab = next_tab_idx;
        }
    }

    fn previous(&mut self) {
        if let Some(tab_idx) = self.cursor.tab {
            if tab_idx > 0 {
                self.cursor.tab = Some(tab_idx - 1);
                return;
            }
        }
        let next_session_idx = self.cursor.session.saturating_sub(1);
        let next_tab_idx = match self.expanded.get(next_session_idx) {
            Some(true) => self.sessions.get(next_session_idx).map(|session| session.tabs.len().saturating_sub(1)),
            Some(false) => None,
            None => None,
        };
        self.cursor.session = next_session_idx;
        self.cursor.tab = next_tab_idx;
    }

    fn next(&mut self) {
        if let Some(session) = self.sessions.get(self.cursor.session) {
            if let Some(tab_idx) = self.cursor.tab {
                if tab_idx + 1 < session.tabs.len() {
                    self.cursor.tab = Some(tab_idx + 1);
                    return;
                }
            }
            let next_session_idx = (self.cursor.session + 1).min(self.sessions.len().saturating_sub(1));
            let next_tab_idx = match self.expanded.get(next_session_idx) {
                Some(true) => self.cursor.tab,
                Some(false) => None,
                None => None,
            };
            self.cursor.session = next_session_idx;
            self.cursor.tab = next_tab_idx;
        }
    }

    pub fn handle_down(&mut self) {
        self.next_with_cycle()
    }

    pub fn handle_up(&mut self) {
        self.previous_with_cycle()
    }

    pub fn handle_left(&mut self)  {
        match self.expanded.get(self.cursor.session) {
            Some(true) => {
                self.toggle(false);
                self.cursor.tab = None;
            }
            Some(false) => {
                self.previous();
            }
            None => {}
        }
    }

    pub fn handle_right(&mut self) {
        match self.expanded.get(self.cursor.session) {
            Some(true) => {
                self.next();
            }
            Some(false) => {
                self.toggle(true);
                self.cursor.tab = Some(0);
            }
            None => {}
        }
    }

    pub fn switch_by_index(&self, index: usize) {
    }

    pub fn switch_to_selected(&self) {
        if let Some(session) = self.sessions.get(self.cursor.session) {
            if let Some(tab_idx) = self.cursor.tab {
                if let Some(tab) = session.tabs.get(tab_idx) {
                    switch_session_with_focus(&session.name, Some(tab.position), None);
                }
            } else if !session.is_current_session {
                switch_session(Some(&session.name));
            }
        };
    }

    pub fn kill_selected(& self) {
        //killing a tab not supported by zellij yet
        if self.cursor.tab.is_some() {
            return;
        }
        if let Some(session) = &self.sessions.get(self.cursor.session) {
            kill_sessions(&[session.name.clone()]);
        }
    }

    pub fn render(&mut self, _rows: usize, _cols: usize) {
        let mut index = 0;
        let mut nested_list = Vec::new();
        for ((session_index, session), is_expanded) in self.sessions.iter().enumerate().zip(self.expanded.iter()) {
            let text = match session.is_current_session {
                true => format!("({0}) {1} (attached)", index, session.name),
                false => format!("({0}) {1}", index, session.name),
            };
            let mut session_line = NestedListItem::new(text).indent(0);
            if !*is_expanded && session_index == self.cursor.session {
                session_line = session_line.selected();
            }
            nested_list.push(session_line);
            index += 1;

            if !*is_expanded {
                continue;
            }

            for (tab_index, tab) in session.tabs.iter().enumerate() {
                let text = format!("({0}) {1}", index, tab.name);
                let mut tab_line = NestedListItem::new(text).indent(1);
                if session_index == self.cursor.session && Some(tab_index) == self.cursor.tab {
                    tab_line = tab_line.selected();
                }
                nested_list.push(tab_line);
                index += 1;
            }
        }
        print_nested_list(nested_list);
    }
}

impl From<Vec<SessionInfo>> for SessionTree {
    fn from(sessions: Vec<SessionInfo>) -> Self {
        Self {
            cursor: Cursor::default(),
            expanded: vec![false; sessions.len()],
            sessions,
        }
    }
}
