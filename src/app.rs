use std::{collections::VecDeque, net::SocketAddr};

use crossterm::event::{KeyEvent, KeyCode};
use tui::{Frame, backend::Backend, style::{Style, Color, Modifier}, widgets::{Block, Paragraph, Wrap, Tabs, Borders}, layout::{Direction, Rect, Constraint, Layout, Alignment}, text::{Spans, Span}};
use tw_econ::connection::Connection;

use crate::{state::{Screen, MainElements, Main, AddConnectionElements, AddConnection}, econtab::EconTab, stringarray::StringArray};

pub struct App {
    current_screen: Screen,
    econ_tabs: VecDeque<EconTab>
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::Welcome,
            econ_tabs: VecDeque::new()
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match &mut self.current_screen {
            Screen::Welcome => if key.code == KeyCode::Enter {
                self.current_screen = Screen::Main(Main::new());
            },
            Screen::Main(data) =>{
                match key.code {
                    KeyCode::Up => data.active = data.active.prev(),
                    KeyCode::Down => data.active = data.active.next(),
                    KeyCode::Esc => self.current_screen = Screen::Exit,

                    _ => match data.active {
                        MainElements::Connections => match key.code {
                            KeyCode::Left => if self.econ_tabs.len() > 0 && data.connection > 0 {
                                data.connection -= 1;
                            }
                            else {
                                data.connection = (self.econ_tabs.len() - 1) as _;
                            },
                            KeyCode::Right => if self.econ_tabs.len() > 0 && data.connection < (self.econ_tabs.len() - 1) as _ {
                                data.connection += 1;
                            }
                            else {
                                data.connection = 0
                            },
                            _ => {}
                        },
                        MainElements::Console => if !self.econ_tabs.is_empty() {
                            let econ_tab = self.econ_tabs.get_mut(data.connection as _).unwrap();
                            match key.code {
                                KeyCode::Left => {
                                    if econ_tab.scroll > 0 {
                                        econ_tab.scroll -= 1;
                                    }
                                },
                                KeyCode::Right => {
                                    if econ_tab.scroll < econ_tab.messages.len() as _ {
                                        econ_tab.scroll += 1;
                                    }
                                },
                                _ => {}
                            }
                        },
                        // todo: replace it with tui_input crate
                        MainElements::Input => if !self.econ_tabs.is_empty() {
                            if let Some(econ_tab) = self.econ_tabs.get_mut(data.connection as _) {
                                match key.code {
                                    KeyCode::Backspace => {
                                        econ_tab.buffer.pop();
                                    },
                                    KeyCode::Char(c) => econ_tab.buffer.push(c),
                                    KeyCode::Enter => {
                                        econ_tab.connection.send(econ_tab.buffer.drain(..).collect::<String>()).unwrap();
                                    }
                                    _ => {}
                                }
                            }
                        },
                        MainElements::Add => match key.code {
                            KeyCode::Enter => {
                                self.current_screen = Screen::AddConnection(AddConnection::new());
                            },
                            _ => {}
                        }
                    }
                }
            },
            Screen::AddConnection(data) => {
                match key.code {
                    KeyCode::Up => data.active = data.active.prev(),
                    KeyCode::Down => data.active = data.active.next(),
                    KeyCode::Esc => self.current_screen = Screen::Main(Main::new()),

                    _ => match data.active {
                        AddConnectionElements::OkButton => match key.code {
                            KeyCode::Enter => {
                                if let Some(econ_tab) = Self::process_connection_data(data.fields) {
                                    self.econ_tabs.push_back(econ_tab);
                                }
                                self.current_screen = Screen::Main(Main::new());
                            },
                            _ => {}
                        },
                        // todo: replace it with tui_input crate
                        _ => match key.code {
                            KeyCode::Backspace => data.fields[data.active as usize].pop(),
                            KeyCode::Char(c) => data.fields[data.active as usize].push(c),
                            _ => {}
                        }
                    }
                }
            },
            Screen::Exit => match key.code {
                KeyCode::Esc => self.current_screen = Screen::Main(Main::new()),
                KeyCode::Enter => return true,
                _ => {}
            },
        }

        false
    }

    pub fn on_tick(&mut self) {
        if !self.econ_tabs.is_empty() {
            let mut dead_connections = Vec::new();
            for (id, econ_tab) in self.econ_tabs.iter().enumerate() {
                if !econ_tab.connection.alive() {
                    dead_connections.push(id);
                }
            }
    
            dead_connections.reverse();
    
            for id in dead_connections {
                self.econ_tabs.remove(id);
            }
    
            for econ_tab in &mut self.econ_tabs {
                // 1 connection = 1 ms
                if let Ok(messages) = econ_tab.connection.recv() {
                    let messages = messages
                        .lines()
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>();
                    econ_tab.messages.extend(messages);
                }
            }
        }
    }

    fn process_connection_data(data: [StringArray<64>; 3]) -> Option<EconTab> {
        let name = data[AddConnectionElements::Name as usize].to_string();
        let address = match data[AddConnectionElements::Address as usize].to_string().parse::<SocketAddr>() {
            Ok(address) => address,
            Err(_) => return None
        };
        let password = data[AddConnectionElements::Password as usize].to_string();

        let mut connection = Connection::new();

        if let Err(_) = connection.launch_with_password(address, password) {
            return None
        }

        let econ_tab = EconTab {
            name,
            connection,
            messages: Vec::new(),
            buffer: String::new(),
            scroll: 0
        };

        Some(econ_tab)
    }
}

pub fn process_app<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // start drawing
    let size = f.size();
    let default_style = Style::default().bg(Color::White).fg(Color::Gray); // todo: add default dark style
    let active_style = Style::default().bg(Color::White).fg(Color::Black);
    let background = Block::default().style(default_style);

    let default_block = Block::default().style(default_style).borders(Borders::ALL);
    let active_block = Block::default().style(active_style).borders(Borders::ALL);

    f.render_widget(background, size);

    match &app.current_screen {
        Screen::Welcome => {
            const WELCOME_TEXT: &str = "tw-econ-tui\n\nThis is a multi-windowed Teeworlds external console.\nYou can freely switch between different connections, like if it was a brand new console shell.\n\nPress Enter to continue";
            let paragraph = Paragraph::new(WELCOME_TEXT)
                .style(active_style)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false });

            f.render_widget(paragraph, centered_rect(30, 50, Rect::new(0, 0, 30, 8), size));
        },
        Screen::Main(data) => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(size);

            let connection_titles = app
                .econ_tabs
                .iter()
                .map(|t| {
                    Spans::from(vec![Span::styled(t.name.clone(), default_style)])
                })
                .collect();

            let highlight_style = if data.active == MainElements::Connections {
                Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black)
                        .fg(Color::White)
            }
            else {
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Gray)
                    .fg(Color::White)
            };

            let mut connections = Tabs::new(connection_titles)
                .block(default_block.clone().title("Connections"))
                .select(data.connection as _)
                .style(default_style)
                .highlight_style(highlight_style);

            let console_scroll = (0, 0);
            let mut console_content = String::new();
            let mut input_content = String::new();

            if let Some(econ_tab) = app.econ_tabs.get(data.connection as _) {
                console_content = econ_tab.messages.join("\n");
                input_content = econ_tab.buffer.clone();
            }
            
            let mut console = Paragraph::new(console_content)
                .style(default_style)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .scroll(console_scroll)
                .block(default_block.clone().title("Console"));

            let mut input = Paragraph::new(input_content)
                .style(default_style)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .block(default_block.clone().title("Input"));

            let mut add = Paragraph::new("Press Enter to add new connection")
                .style(default_style)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false })
                .block(default_block.clone().title("Add"));

            match data.active {
                MainElements::Connections => {
                    connections = connections
                        .style(active_style)
                        .block(active_block.clone()
                        .title("Connections"));
                },
                MainElements::Console => {
                    console = console
                        .style(active_style)
                        .block(active_block.clone()
                        .title("Console"));
                },
                MainElements::Input => {
                    input = input
                        .style(active_style)
                        .block(active_block.clone()
                        .title("Input"));
                },
                MainElements::Add => {
                    add = add
                        .style(active_style)
                        .block(active_block.clone()
                        .title("Add"));
                },
            }

            f.render_widget(connections, chunks[0]);
            f.render_widget(console, chunks[1]);
            f.render_widget(input, chunks[2]);
            f.render_widget(add, chunks[3]);
        },
        Screen::AddConnection(data) => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20),
                        Constraint::Percentage(20)
                    ].as_ref()
                )
                .split(
                    centered_rect(
                        30,
                        40,
                        Rect::new(0, 0, 12, 20),
                        size
                    )
                );

                let title = Paragraph::new("Connection Data")
                        .alignment(Alignment::Center)
                        .style(active_style);

                f.render_widget(title, chunks[0]);

                let block_names = vec!["Name", "Address", "Password"];
                
                for i in 0..3 {
                    let mut field_style = default_style;
                    let mut block = default_block.clone().title(block_names[i]);

                    if data.active as usize == i {
                        field_style = active_style;
                        block = block.style(active_style);
                    }

                    let field = Paragraph::new(data.fields[i].to_string())
                        .alignment(Alignment::Center)
                        .style(field_style)
                        .block(block.clone().title(block_names[i]));

                    f.render_widget(field, chunks[i + 1]);
                }

                let ok_style = if data.active == AddConnectionElements::OkButton {
                    active_style
                }
                else {
                    default_style
                };

                let ok_button = Paragraph::new("Add")
                    .style(ok_style)
                    .alignment(Alignment::Center);

                f.render_widget(ok_button, chunks[4]);
        },
        Screen::Exit => {
            const EXIT_TEXT: &str = "Are you sure you want to quit?\n\nPress Enter to quit\nPress Escape to cancel";
            let paragraph = Paragraph::new(EXIT_TEXT)
                .style(active_style)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false });

            f.render_widget(paragraph, centered_rect(30, 50, Rect::new(0, 0, 30, 8), size));
        },
    }

}

fn centered_rect(percent_x: u16, percent_y: u16, min_r: Rect, base_r: Rect) -> Rect {
    let min_percent_x = ((min_r.width.min(base_r.width) as f32) / (base_r.width as f32) * 100f32) as u16;
    let min_percent_y = ((min_r.height.min(base_r.height) as f32) / (base_r.height as f32) * 100f32) as u16;

    let percent_x = percent_x.max(min_percent_x);
    let percent_y = percent_y.max(min_percent_y);

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(base_r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
