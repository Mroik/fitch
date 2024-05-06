use crate::fitch::Fitch;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use std::io::{stdout, Stdout};

const INFO_AREA_HEIGHT: u16 = 3;

pub struct Renderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Renderer {
    fn new() -> std::io::Result<Renderer> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut renderer = Renderer {
            terminal: Terminal::new(CrosstermBackend::new(stdout()))?,
        };
        renderer.terminal.clear()?;
        Ok(renderer)
    }

    fn render_fitch(&mut self, model: &Fitch) {
        self.terminal
            .draw(|frame| {
                let (f_a, i_a) = base_area(frame.size());
                let fitch_widget = Paragraph::new(model.to_string()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                );
                let info_widget = Paragraph::new(info_text());

                frame.render_widget(fitch_widget, f_a);
                frame.render_widget(info_widget, i_a);
            })
            .unwrap();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}

fn info_text() -> String {
    // TODO fill info array
    let info = [""];
    info.iter().fold(String::new(), |mut acc, inf| {
        acc.push_str(inf);
        acc
    })
}

fn base_area(whole: Rect) -> (Rect, Rect) {
    let temp = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(whole.height - INFO_AREA_HEIGHT),
            Constraint::Length(INFO_AREA_HEIGHT),
        ])
        .split(whole);

    (temp[0], temp[1])
}

fn expression_box_area(whole: Rect) -> Rect {
    let temp = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(whole.height / 2 - 1),
            Constraint::Length(1),
            Constraint::Length(whole.height / 2),
        ])
        .split(whole)[1];
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(18),
            Constraint::Percentage(64),
            Constraint::Percentage(18),
        ])
        .split(temp)[1]
}
