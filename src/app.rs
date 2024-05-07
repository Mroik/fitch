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
        };
        app.render();
        Ok(app)
    }

    fn render(&mut self) {
        let (title, render_box) = match self.state {
            State::AddAssumption => ("Assumption expression", true),
            State::AddSubproof => ("Subproof expression", true),
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
                    _ => todo!(),
                }
            }

            self.render();
            self.info_buffer.clear();
        }
    }

    fn listen_add_subproof(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App, result: parser::Result| match result {
            parser::Result::Failure => app_context
                .info_buffer
                .push_str("Expression entered is invalid"),
            parser::Result::Success(expr, _) => {
                app_context.model.add_subproof(&expr);
                app_context.state = State::Noraml;
                app_context.expression_buffer.clear();
                app_context.expression_cursor = 0;
            }
        };

        self.handle_expression_box_event(code, handler);
    }

    fn listen_add_assumption(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App, result: parser::Result| match result {
            parser::Result::Failure => app_context
                .info_buffer
                .push_str("Expression entered is invalid"),
            parser::Result::Success(expr, _) => {
                app_context.model.add_assumption(&expr);
                app_context.state = State::Noraml;
                app_context.expression_buffer.clear();
                app_context.expression_cursor = 0;
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

    // Not sure if it avoids much code duplication. It also requires a clone thus another
    // allocation. If it starts to become bothersome just revert this code section to
    // 103cd74bd33e8bb550512cb745b2ff6bf89e35e1
    fn handle_expression_box_event(
        &mut self,
        code: &KeyCode,
        mut handler: impl FnMut(&mut App, parser::Result),
    ) {
        match code {
            KeyCode::Enter => {
                let buf = self.expression_buffer.clone();
                let res = parse_expression(&buf);
                handler(self, res);
            }
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
            State::Noraml => [
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
            State::AddAssumption => self.info_buffer.clone(),
            State::AddSubproof => self.info_buffer.clone(),
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
