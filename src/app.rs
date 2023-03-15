use std::collections::HashMap;
use std::net::SocketAddr;

use crossterm::event::{KeyEvent, KeyCode};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};

use tw_recon::connection::Connection;

pub enum State {
    WelcomeBlock,
    ConnectionsBlock(Option<usize>), // choosen connection
    NewConnectionBlock(EditingState),
}


pub enum EditingState {
    Nothing,
    Name,
    Address,
    Pass
}

type Econ = Connection<2048, 16>;

pub struct App {
    connections: Vec<(Econ, String)>,
    connection_name: String,
    connection_addr: String,
    connection_pass: String,
    state: State,
    current_connection: Option<String>
}

impl App {
    pub fn new() -> Self {
        App {
            connections: Vec::new(),
            connection_name: String::new(),
            connection_addr: String::new(),
            connection_pass: String::new(),
            state: State::WelcomeBlock,
            current_connection: None
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match &self.state {
            State::WelcomeBlock => {
                match key.code {
                    KeyCode::Enter => {
                        self.state = State::ConnectionsBlock(None)
                    },
                    _ => {}
                }
            },
            State::ConnectionsBlock(id) => {
                match key.code {
                    KeyCode::Enter => {
                        if id == &self.current_connection {
                            if let Some(id) = id {
                                let (connection, buffer) = self.connections.get_mut(id).unwrap();
                                if !buffer.is_empty() {
                                    connection.send(buffer.to_string()).unwrap(); // todo: handle send errors
                                }
                            }
                        }
                        else {
                            self.current_connection = id;
                        }
                    },
                    _ => {}
                }
            },
            State::NewConnectionBlock(state) => todo!(),
        }
    }
}

pub fn process_app<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(size);
}
