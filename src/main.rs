mod config;
mod pane;
mod session;
mod sessiontree;
mod tab;
mod utils;

use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use config::Config;
use sessiontree::SessionTree;

#[derive(Default)]
struct State {
    session_tree: SessionTree,
    initialised: bool,
    config: Config,

    handling_sessionpick_request_from: Option<(PipeSource, BTreeMap<String, String>)>,
    debug: String,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from(configuration);
        request_permission(&[
            PermissionType::ChangeApplicationState,
            PermissionType::MessageAndLaunchOtherPlugins,
            PermissionType::ReadApplicationState,
            PermissionType::ReadCliPipes,
            PermissionType::RunCommands,
        ]);
        subscribe(&[EventType::SessionUpdate, EventType::Key]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::SessionUpdate(sessions, _) => {
                if !self.initialised {
                    self.session_tree = SessionTree::new(sessions, &self.config);
                    self.initialised = true;
                    should_render = true;
                }
            }
            Event::Key(key) => {
                match key {
                    // Select the node under the cursor
                    KeyWithModifier {
                        bare_key: BareKey::Enter,
                        key_modifiers: _,
                    } => {
                        let _ = match self.handling_sessionpick_request_from {
                            Some(_) => self.handle_sessionpick_request(),
                            _ => self.session_tree.switch_to_selected(),
                        };
                    }
                    // Move up, looping around
                    KeyWithModifier {
                        bare_key: BareKey::Char('k') | BareKey::Up,
                        key_modifiers: _,
                    } => {
                        let _ = self.session_tree.handle_up();
                        should_render = true;
                    }
                    // Move down, looping around
                    KeyWithModifier {
                        bare_key: BareKey::Char('j') | BareKey::Down,
                        key_modifiers: _,
                    } => {
                        let _ = self.session_tree.handle_down();
                        should_render = true;
                    }
                    // Collapse the current node, moving up if already collapsed
                    KeyWithModifier {
                        bare_key: BareKey::Char('h') | BareKey::Left,
                        key_modifiers: _,
                    } => {
                        let _ = self.session_tree.handle_left();
                        should_render = true;
                    }
                    // Expand the current node, moving down if already expanded
                    KeyWithModifier {
                        bare_key: BareKey::Char('l') | BareKey::Right,
                        key_modifiers: _,
                    } => {
                        let _ = self.session_tree.handle_right();
                        should_render = true;
                    }
                    // Kill the current node
                    KeyWithModifier {
                        bare_key: BareKey::Char('x') | BareKey::Delete,
                        key_modifiers: _,
                    } => {
                        let _ = self.session_tree.kill_selected();
                        self.initialised = false;
                        should_render = true;
                    }
                    // Select the node at the given index
                    KeyWithModifier {
                        bare_key: BareKey::Char(c),
                        key_modifiers: _,
                    } => {
                        if let Some(digit) = c.to_digit(10) {
                            let _ = self.session_tree.switch_by_index(digit as usize);
                        }
                        // Capital letters are used to select the node at the given index
                        else if c.is_ascii_uppercase() {
                            let index = 10 + c as u8 - b'A';
                            let _ = self.session_tree.switch_by_index(index as usize);
                        }
                        //hide_self();
                        should_render = true;
                    }
                    // Quit
                    KeyWithModifier {
                        bare_key: BareKey::Esc,
                        key_modifiers: _,
                    } => {
                        hide_self();
                    }
                    _ => (),
                }
            }
            _ => (),
        };
        should_render
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if pipe_message.is_private && pipe_message.name == "sessionpicker" {
            if let PipeSource::Cli(pipe_id) = &pipe_message.source {
                self.debug = format!("Received sessionpicker request from cli pipe {}", pipe_id);
                block_cli_pipe_input(pipe_id);
            }
            self.handling_sessionpick_request_from = Some((pipe_message.source, pipe_message.args));
            true
        } else {
            false
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        println!();
        if !self.debug.is_empty() {
            println!("{}", self.debug);
            println!();
        }
        self.session_tree.render(rows.saturating_sub(3), cols);
    }
}

impl State {
    fn handle_sessionpick_request(&mut self) -> Result<(), String> {
        let current_node = self.session_tree.get_current_node()?;
        let session = self
            .session_tree
            .get_session(current_node.borrow().index())?;
        let response = session.borrow().identifier();

        match &self.handling_sessionpick_request_from {
            Some((PipeSource::Plugin(plugin_id), args)) => {
                pipe_message_to_plugin(
                    MessageToPlugin::new("sessionpicker_result")
                        .with_destination_plugin_id(*plugin_id)
                        .with_args(args.clone())
                        .with_payload(response),
                );
                close_self();
            }
            Some((PipeSource::Cli(pipe_id), _args)) => {
                cli_pipe_output(pipe_id, &response);
                unblock_cli_pipe_input(pipe_id);
                close_self();
            }
            _ => {}
        };
        Ok(())
    }
}
