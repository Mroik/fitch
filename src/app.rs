use crate::{
    fitch::Fitch,
    parser::{self, parse_expression},
    ui::Renderer,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub struct App {
    model: Fitch,
    renderer: Renderer,
    state: State,
    expression_buffer: String,
    expression_cursor: u16,
    info_buffer: String,
    warning: bool,
}

impl App {
    pub fn new() -> std::io::Result<App> {
        let mut app = App {
            model: Fitch::new(),
            renderer: Renderer::new()?,
            state: State::Noraml,
            expression_buffer: String::new(),
            expression_cursor: 0,
            info_buffer: String::new(),
            warning: false,
        };
        app.render();
        Ok(app)
    }

    fn render(&mut self) {
        let (title, render_box) = match self.state {
            State::AddAssumption => ("Assumption expression", true),
            State::AddSubproof => ("Subproof expression", true),
            State::AbsurdumState(_) => ("Assumption index", true),
            _ => ("", false),
        };
        self.renderer.render(
            &self.model,
            &self.info_text(),
            title,
            &self.expression_buffer,
            self.expression_cursor,
            render_box,
        );
    }

    pub fn listen(&mut self) {
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
                    State::AddAssumption => self.listen_add_assumption(&key.code),
                    State::AddSubproof => self.listen_add_subproof(&key.code),
                    State::IntroduceChoice => self.listen_introduce(&key.code),
                    State::AbsurdumState(_) => self.listen_absurdum(&key.code),
                    _ => todo!(),
                }
            }

            self.render();
            self.info_buffer.clear();
            self.warning = false;
        }
    }

    fn listen_absurdum(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| match app_context.state {
            State::AbsurdumState(AbsurdumState::IntroduceGetAssumption1) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(v) => {
                        app_context.state =
                            State::AbsurdumState(AbsurdumState::IntroduceGetAssumption2(v));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::AbsurdumState(AbsurdumState::IntroduceGetAssumption2(a1)) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(a2) => {
                        if !app_context.model.introduce_absurdum(a1, a2) {
                            app_context
                                .info_buffer
                                .push_str("The input index are not valid");
                            app_context.warning = true;
                        }
                        app_context.reset_expression_box();
                        app_context.state = State::Noraml;
                    }
                }
            }
            // TODO Eliminate absurdum
            _ => (),
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_introduce(&mut self, code: &KeyCode) {
        match code {
            KeyCode::Char('a') => {
                self.state = State::AbsurdumState(AbsurdumState::IntroduceGetAssumption1)
            }
            _ => (),
        }
    }

    fn listen_add_subproof(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| {
            let buf = app_context.expression_buffer.clone();
            match parse_expression(&buf) {
                parser::Result::Failure => app_context
                    .info_buffer
                    .push_str("Expression entered is invalid"),
                parser::Result::Success(expr, _) => {
                    app_context.model.add_subproof(&expr);
                    app_context.state = State::Noraml;
                    app_context.reset_expression_box();
                }
            }
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_add_assumption(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| {
            let buf = app_context.expression_buffer.clone();
            match parse_expression(&buf) {
                parser::Result::Failure => app_context
                    .info_buffer
                    .push_str("Expression entered is invalid"),
                parser::Result::Success(expr, _) => {
                    app_context.model.add_assumption(&expr);
                    app_context.state = State::Noraml;
                    app_context.reset_expression_box();
                }
            }
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_normal(&mut self, code: &KeyCode) {
        match code {
            KeyCode::Char('i') => self.state = State::IntroduceChoice,
            KeyCode::Char('e') => self.state = State::EliminateChoice,
            KeyCode::Char('a') => self.state = State::AddAssumption,
            KeyCode::Char('s') => self.state = State::AddSubproof,
            KeyCode::Char('n') => self.model.end_subproof(),
            KeyCode::Char('d') => self.model.delete_last_row(),
            KeyCode::Char('q') => self.state = State::Quit,
            _ => (),
        }
    }

    fn reset_expression_box(&mut self) {
        self.expression_buffer.clear();
        self.expression_cursor = 0;
    }

    // Not sure if it avoids much code duplication. It also requires a clone thus another
    // allocation. If it starts to become bothersome just revert this code section to
    // 103cd74bd33e8bb550512cb745b2ff6bf89e35e1
    fn handle_expression_box_event(&mut self, code: &KeyCode, mut handler: impl FnMut(&mut App)) {
        match code {
            KeyCode::Enter => handler(self),
            KeyCode::Backspace if !self.expression_buffer.is_empty() => {
                if self.expression_cursor > 0 {
                    self.expression_buffer
                        .remove(self.expression_cursor as usize - 1);
                    self.expression_cursor -= 1;
                }
            }
            KeyCode::Char(c) => {
                self.expression_buffer.push(*c);
                self.expression_cursor += 1;
            }
            KeyCode::Esc => {
                self.expression_buffer.clear();
                self.state = State::Noraml
            }
            KeyCode::Left if self.expression_cursor > 0 => self.expression_cursor -= 1,
            KeyCode::Right if (self.expression_cursor as usize) < self.expression_buffer.len() => {
                self.expression_cursor += 1
            }
            _ => (),
        }
    }

    fn info_text(&self) -> String {
        match self.state {
            State::Noraml if !self.warning => [
                "[i]ntroduce",
                "[e]liminate",
                "add [a]ssumption",
                "add [s]ubproof",
                "e[n]d subproof",
                "[d]elete last row",
                "[q]uit",
            ]
            .join("   ")
            .to_string(),
            State::IntroduceChoice => {
                ["[a]bsurdum", "a[n]d", "[o]r", "no[t]", "[i]mplies", "i[f]f"]
                    .join("    ")
                    .to_string()
            }
            _ => self.info_buffer.clone(),
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
    EliminateGetAssumption,
    EliminateGetProposition(usize),
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
