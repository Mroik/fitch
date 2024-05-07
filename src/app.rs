use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{fitch::Fitch, ui::Renderer};

struct App {
    model: Fitch,
    renderer: Renderer,
    state: State,
    expression_buffer: String,
}

impl App {
    fn new() -> std::io::Result<App> {
        let mut app = App {
            model: Fitch::new(),
            renderer: Renderer::new()?,
            state: State::Noraml,
            expression_buffer: String::new(),
        };
        app.render();
        Ok(app)
    }

    fn render(&mut self) {
        self.renderer.render_fitch(&self.model, &self.info_text());
        match self.state {
            State::Noraml => self.renderer.render_expression_box(&self.expression_buffer),
            _ => (),
        }
    }

    fn listen(&mut self) {
        loop {
            if self.state == State::Quit {
                break;
            }

            if let Event::Key(key) = event::read().unwrap() {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match self.state {
                    State::Noraml => self.listen_normal(&key.code),
                    _ => todo!(),
                }
            }

            self.render();
        }
    }

    fn listen_normal(&mut self, code: &KeyCode) {
        match code {
            KeyCode::Char('i') => self.state = State::IntroduceChoice,
            KeyCode::Char('e') => self.state = State::EliminateChoice,
            KeyCode::Char('a') => self.state = State::AddAssumption,
            KeyCode::Char('s') => self.state = State::AddSubproof,
            KeyCode::Char('q') => self.state = State::Quit,
            _ => (),
        }
    }

    fn info_text(&self) -> String {
        match self.state {
            State::Noraml => [
                "[i]ntroduce",
                "[e]liminate",
                "add [a]ssumption",
                "add [s]ubproof",
                "[d]elete last row",
                "[q]uit",
            ]
            .join("   ")
            .to_string(),
            _ => "".to_string(),
        }
    }
}

#[derive(PartialEq)]
enum State {
    Noraml,
    IntroduceChoice,
    EliminateChoice,
    AddAssumption,
    AddSubproof,
    AbsurdumState(AbsurdumState),
    AndState(AndState),
    OrState(OrState),
    NotState(NotState),
    ImpliesState(ImpliesState),
    IffState(IffState),
    Quit,
}

#[derive(PartialEq)]
enum AbsurdumState {
    IntroduceGetAssumption1,
    IntroduceGetAssumption2(usize),
}

#[derive(PartialEq)]
enum AndState {
    IntroduceGetLeftAssumption,
    IntroduceGetRightAssumption(usize),
    EliminateGetAssumption,
    EliminateGetProposition(usize),
}

#[derive(PartialEq)]
enum OrState {
    IntroduceGetAssumption,
    IntroduceGetProposition(usize),
    EliminateGetAssumption,
    EliminateGetLeftSubproof(usize),
    EliminateGetRightSubproof(usize, usize),
}

#[derive(PartialEq)]
enum NotState {
    Introduce,
    Eliminate,
}

#[derive(PartialEq)]
enum ImpliesState {
    Introduce,
    EliminateGetAssumption,
    EliminateGetLeft(usize),
}

#[derive(PartialEq)]
enum IffState {
    IntroduceGetLeftSubproof,
    IntroduceGetRightSubproof(usize),
    EliminateGetAssumption,
    EliminateGetTruth(usize),
}
