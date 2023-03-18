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


pub type EconId = u8;
pub struct EconTab {
    pub connection: Connection<2048, 16>,
    pub messages: Vec<String>,
    pub buffer: String
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    WelcomeBlock,
    ConnectionsBlock(Option<EconId>), // choosen connection
    NewConnectionBlock(NewConnection),
}

#[derive(Debug, Clone, Copy)]
pub enum NewConnectionState {
    Idle,
    Editing(NewConnectionField)
}

#[derive(Debug, Clone, Copy)]
pub enum NewConnectionField {
    Name = 0,
    Address,
    Password,
    Num
}

impl From<u8> for NewConnectionField {
    fn from(value: u8) -> Self {
        match value {
            0 => NewConnectionField::Name,
            1 => NewConnectionField::Address,
            2 => NewConnectionField::Password,
            _ => NewConnectionField::Name
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StringArray<const MAX_LENGTH: usize> {
    pub len: usize,
    pub array: [char; MAX_LENGTH]
}

impl<const MAX_LENGTH: usize> StringArray<MAX_LENGTH> {
    pub fn new() -> Self {
        Self { len: 0, array: ['0'; MAX_LENGTH] }
    }

    pub fn push(&mut self, c: char) {
        if self.len < MAX_LENGTH {
            self.array[self.len] = c;
            self.len += 1;
        }
    }

    pub fn pop(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    pub fn to_string(&self) -> String {
        let result: String = self.array[..self.len].iter().collect();

        result
    }
}

pub type StringField = StringArray<{64 * 4}>;

#[derive(Debug, Clone, Copy)]
pub struct NewConnection {
    pub state: NewConnectionState,
    pub fields: [StringField; NewConnectionField::Num as usize]
}

pub struct App {
    connections: HashMap<EconId, EconTab>,
    state: State,
    current_connection: Option<EconId>
}

impl App {
    pub fn new() -> Self {
        App {
            connections: HashMap::new(),
            state: State::WelcomeBlock,
            current_connection: None
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match self.state {
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
                        // if user choosen tab 
                        if id == self.current_connection {
                            if let Some(id) = id {
                                let econ = self.connections.get_mut(&id).unwrap();

                                // if we have something in buffer we should send it on enter key
                                if !econ.buffer.is_empty() {
                                    econ.connection.send(econ.buffer.to_string()).unwrap(); // todo: handle send errors
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
            State::NewConnectionBlock(mut connection) => {
                match connection.state {
                    NewConnectionState::Idle => {
                        match key.code {
                            KeyCode::Tab => {
                                connection.state = NewConnectionState::Editing(NewConnectionField::Name);
                            },
                            KeyCode::Enter => {
                                let id = self.add_connection(&connection);
                                self.state = State::ConnectionsBlock(id);
                            },
                            KeyCode::Esc => {
                                self.state = State::ConnectionsBlock(self.current_connection);
                            },
                            _ => {}
                        }
                    },
                    NewConnectionState::Editing(field) => {
                        match key.code {
                            KeyCode::Tab => {
                                connection.state = NewConnectionState::Editing(((field as u8 + 1) % NewConnectionField::Num as u8).into());
                            },
                            KeyCode::Enter => {
                                let id = self.add_connection(&connection);
                                self.state = State::ConnectionsBlock(id);
                            },
                            KeyCode::Char(c) => {
                                connection.fields[field as usize].push(c);
                            }
                            KeyCode::Backspace => {
                                connection.fields[field as usize].pop();
                            }
                            KeyCode::Esc => {
                                connection.state = NewConnectionState::Idle;
                            },
                            _ => {}
                        }
                    }
                }
            },
        }
    }

    fn add_connection(&mut self, connection: &NewConnection) -> Option<EconId> {
        let mut econ: Connection<2048, 16> = Connection::new();
        let result = econ.launch_with_password(
            connection.fields[NewConnectionField::Address as usize].to_string().parse::<SocketAddr>().unwrap(),
            connection.fields[NewConnectionField::Password as usize].to_string());

        if let Err(_error) = result {
            return None;
        }

        let econ_tab = EconTab {
            connection: econ,
            messages: Vec::new(),
            buffer: String::new()
        };

        for i in 0..255u8 {
            if self.connections.contains_key(&i) {
                continue;
            }

            self.connections.insert(i, econ_tab);
        }

        None
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
