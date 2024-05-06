use std::io::Stdout;

use ratatui::{backend::CrosstermBackend, Terminal};

pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}
