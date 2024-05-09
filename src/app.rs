use crate::{
    fitch::Fitch,
    parser::{self, parse_expression},
    state::{AbsurdumState, AndState, IffState, ImpliesState, NotState, OrState, State},
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
            State::AbsurdumState(AbsurdumState::EliminateGetProposition(_))
            | State::OrState(OrState::IntroduceGetProposition(_)) => {
                ("Expression to introduce", true)
            }
            State::AbsurdumState(_)
            | State::AndState(AndState::IntroduceGetLeftAssumption)
            | State::AndState(AndState::IntroduceGetRightAssumption(_))
            | State::OrState(OrState::IntroduceGetAssumption)
            | State::NotState(_) => ("Assumption index", true),
            State::AndState(AndState::EliminateGetAssumption)
            | State::OrState(OrState::EliminateGetAssumption) => {
                ("And expression to eliminate", true)
            }
            State::AndState(AndState::EliminateGetProposition(_)) => {
                ("And assumption to use", true)
            }
            State::OrState(OrState::EliminateGetLeftSubproof(_))
            | State::OrState(OrState::EliminateGetRightSubproof(_, _))
            | State::ImpliesState(ImpliesState::Introduce)
            | State::IffState(IffState::IntroduceGetLeftSubproof)
            | State::IffState(IffState::IntroduceGetRightSubproof(_)) => ("Subproof to use", true),
            State::Reiterate => ("Select proposition to reiterate", true),
            State::ImpliesState(ImpliesState::EliminateGetAssumption)
            | State::IffState(IffState::EliminateGetAssumption) => {
                ("Implication to eliminate", true)
            }
            State::ImpliesState(ImpliesState::EliminateGetLeft(_))
            | State::IffState(IffState::EliminateGetTruth(_)) => ("Index of the truth", true),
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
                    State::Reiterate => self.listen_reiterate(&key.code),
                    State::IntroduceChoice => self.listen_introduce(&key.code),
                    State::EliminateChoice => self.listen_eliminate(&key.code),
                    State::AbsurdumState(_) => self.listen_absurdum(&key.code),
                    State::AndState(_) => self.listen_and(&key.code),
                    State::OrState(_) => self.listen_or(&key.code),
                    State::NotState(_) => self.listen_not(&key.code),
                    State::ImpliesState(_) => self.listen_implies(&key.code),
                    State::IffState(_) => self.listen_iff(&key.code),
                    _ => todo!(),
                }
            }

            self.render();
            self.info_buffer.clear();
            self.warning = false;
        }
    }

    fn listen_iff(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| match app_context.state {
            State::IffState(IffState::IntroduceGetLeftSubproof) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(left) => {
                        app_context.state =
                            State::IffState(IffState::IntroduceGetRightSubproof(left));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::IffState(IffState::IntroduceGetRightSubproof(left)) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(right) => {
                        if !app_context.model.introduce_iff(left, right) {
                            app_context
                                .info_buffer
                                .push_str("Select the left subproof then the right subproof");
                            app_context.warning = true;
                        }
                        app_context.state = State::Noraml;
                        app_context.reset_expression_box();
                    }
                }
            }
            State::IffState(IffState::EliminateGetAssumption) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(to_elim) => {
                        app_context.state = State::IffState(IffState::EliminateGetTruth(to_elim));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::IffState(IffState::EliminateGetTruth(to_elim)) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(truth) => {
                        if !app_context.model.eliminate_iff(to_elim, truth) {
                            app_context.info_buffer.push_str(
                                "Choose the double implication to eliminate then the truth",
                            );
                            app_context.warning = true;
                        }
                        app_context.state = State::Noraml;
                        app_context.reset_expression_box();
                    }
                }
            }
            _ => (),
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_implies(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| match app_context.state {
            State::ImpliesState(ImpliesState::Introduce) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(subproof) => {
                        if !app_context.model.introduce_implies(subproof) {
                            app_context.info_buffer.push_str("Invalid subproof");
                            app_context.warning = true;
                        }
                        app_context.state = State::Noraml;
                        app_context.reset_expression_box();
                    }
                }
            }
            State::ImpliesState(ImpliesState::EliminateGetAssumption) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(to_elim) => {
                        app_context.state =
                            State::ImpliesState(ImpliesState::EliminateGetLeft(to_elim));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::ImpliesState(ImpliesState::EliminateGetLeft(to_elim)) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(truth) => {
                        if !app_context.model.eliminate_implies(to_elim, truth) {
                            app_context
                                .info_buffer
                                .push_str("Choose the implication to eliminate then the truth");
                            app_context.warning = true;
                        }
                        app_context.state = State::Noraml;
                        app_context.reset_expression_box();
                    }
                }
            }
            _ => (),
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_not(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| match app_context.state {
            State::NotState(NotState::Introduce) => match app_context.expression_buffer.parse() {
                Err(_) => {
                    app_context
                        .info_buffer
                        .push_str("The input value is not a valid index");
                }
                Ok(index) => {
                    if !app_context.model.introduce_not(index) {
                        app_context
                            .info_buffer
                            .push_str("Subproof assumption does not generate an absurdum");
                        app_context.warning = true;
                    }
                    app_context.state = State::Noraml;
                    app_context.reset_expression_box();
                }
            },
            State::NotState(NotState::Eliminate) => match app_context.expression_buffer.parse() {
                Err(_) => {
                    app_context
                        .info_buffer
                        .push_str("The input value is not a valid index");
                }
                Ok(index) => {
                    if !app_context.model.eliminate_not(index) {
                        app_context
                            .info_buffer
                            .push_str("Expression selected is not a double negation");
                        app_context.warning = true;
                    }
                    app_context.state = State::Noraml;
                    app_context.reset_expression_box();
                }
            },
            _ => (),
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_reiterate(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| match app_context.expression_buffer.parse() {
            Err(_) => {
                app_context
                    .info_buffer
                    .push_str("The input value is not a valid index");
            }
            Ok(i) => {
                if !app_context.model.reiterate(i) {
                    app_context.info_buffer.push_str("Invalid index");
                    app_context.warning = true;
                }
                app_context.state = State::Noraml;
                app_context.reset_expression_box();
            }
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_or(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| match app_context.state {
            State::OrState(OrState::IntroduceGetAssumption) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(ass) => {
                        app_context.state = State::OrState(OrState::IntroduceGetProposition(ass));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::OrState(OrState::IntroduceGetProposition(ass)) => {
                match parse_expression(&app_context.expression_buffer) {
                    parser::Result::Failure => app_context
                        .info_buffer
                        .push_str("The input expression is not valid"),
                    parser::Result::Success(ris, _) => {
                        if !app_context.model.introduce_or(ass, &ris) {
                            app_context.info_buffer.push_str(
                                "Select the left or right prop used in the input expression",
                            );
                            app_context.warning = true;
                        }
                        app_context.state = State::Noraml;
                        app_context.reset_expression_box();
                    }
                }
            }
            State::OrState(OrState::EliminateGetAssumption) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(ass) => {
                        app_context.state = State::OrState(OrState::EliminateGetLeftSubproof(ass));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::OrState(OrState::EliminateGetLeftSubproof(ass)) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(left) => {
                        app_context.state =
                            State::OrState(OrState::EliminateGetRightSubproof(ass, left));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::OrState(OrState::EliminateGetRightSubproof(ass, left)) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(right) => {
                        if !app_context.model.eliminate_or(ass, left, right) {
                            app_context
                                .info_buffer
                                .push_str("Select the or to eliminate then the 2 subproofs");
                            app_context.warning = true;
                        }
                        app_context.state = State::Noraml;
                        app_context.reset_expression_box();
                    }
                }
            }
            _ => (),
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_and(&mut self, code: &KeyCode) {
        let handler = |app_context: &mut App| match app_context.state {
            State::AndState(AndState::IntroduceGetLeftAssumption) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(left) => {
                        app_context.state =
                            State::AndState(AndState::IntroduceGetRightAssumption(left));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::AndState(AndState::IntroduceGetRightAssumption(left)) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(right) => {
                        if !app_context.model.introduce_and(left, right) {
                            app_context
                                .info_buffer
                                .push_str("Selected assumptions are not valid");
                            app_context.warning = true;
                        }
                        app_context.state = State::Noraml;
                        app_context.reset_expression_box();
                    }
                }
            }
            State::AndState(AndState::EliminateGetAssumption) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(left) => {
                        app_context.state =
                            State::AndState(AndState::EliminateGetProposition(left));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::AndState(AndState::EliminateGetProposition(left)) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(right) => {
                        if !app_context.model.eliminate_and(left, right) {
                            app_context.info_buffer.push_str(
                                "Select first the AND statement to eliminate, then the assumption",
                            );
                            app_context.warning = true;
                        }
                        app_context.state = State::Noraml;
                        app_context.reset_expression_box();
                    }
                }
            }
            _ => unreachable!(),
        };
        self.handle_expression_box_event(code, handler);
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
            State::AbsurdumState(AbsurdumState::EliminateGetAssumption) => {
                match app_context.expression_buffer.parse() {
                    Err(_) => {
                        app_context
                            .info_buffer
                            .push_str("The input value is not a valid index");
                    }
                    Ok(assum) => {
                        app_context.state =
                            State::AbsurdumState(AbsurdumState::EliminateGetProposition(assum));
                        app_context.reset_expression_box();
                    }
                }
            }
            State::AbsurdumState(AbsurdumState::EliminateGetProposition(assum)) => {
                match parse_expression(&app_context.expression_buffer) {
                    parser::Result::Failure => app_context
                        .info_buffer
                        .push_str("Expression entered is invalid"),
                    parser::Result::Success(ded, _) => {
                        if !app_context.model.eliminate_absurdum(assum, &ded) {
                            app_context
                                .info_buffer
                                .push_str("The assumption you selected is not an absurdum");
                            app_context.warning = true;
                        }
                        app_context.reset_expression_box();
                        app_context.state = State::Noraml;
                    }
                }
            }
            _ => unreachable!(),
        };
        self.handle_expression_box_event(code, handler);
    }

    fn listen_eliminate(&mut self, code: &KeyCode) {
        match code {
            KeyCode::Esc => self.state = State::Noraml,
            KeyCode::Char('a') => {
                self.state = State::AbsurdumState(AbsurdumState::EliminateGetAssumption)
            }
            KeyCode::Char('n') => self.state = State::AndState(AndState::EliminateGetAssumption),
            KeyCode::Char('o') => self.state = State::OrState(OrState::EliminateGetAssumption),
            KeyCode::Char('t') => self.state = State::NotState(NotState::Eliminate),
            KeyCode::Char('i') => {
                self.state = State::ImpliesState(ImpliesState::EliminateGetAssumption)
            }
            KeyCode::Char('f') => self.state = State::IffState(IffState::EliminateGetAssumption),
            _ => (),
        }
    }

    fn listen_introduce(&mut self, code: &KeyCode) {
        match code {
            KeyCode::Esc => self.state = State::Noraml,
            KeyCode::Char('a') => {
                self.state = State::AbsurdumState(AbsurdumState::IntroduceGetAssumption1)
            }
            KeyCode::Char('n') => {
                self.state = State::AndState(AndState::IntroduceGetLeftAssumption)
            }
            KeyCode::Char('o') => self.state = State::OrState(OrState::IntroduceGetAssumption),
            KeyCode::Char('t') => self.state = State::NotState(NotState::Introduce),
            KeyCode::Char('i') => self.state = State::ImpliesState(ImpliesState::Introduce),
            KeyCode::Char('f') => self.state = State::IffState(IffState::IntroduceGetLeftSubproof),
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
            KeyCode::Char('r') => self.state = State::Reiterate,
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
                self.expression_buffer
                    .insert(self.expression_cursor as usize, *c);
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
                "[r]eiterate",
                "[d]elete last row",
                "[q]uit",
            ]
            .join("   ")
            .to_string(),
            State::IntroduceChoice | State::EliminateChoice => {
                ["[a]bsurdum", "a[n]d", "[o]r", "no[t]", "[i]mplies", "i[f]f"]
                    .join("    ")
                    .to_string()
            }
            _ => self.info_buffer.clone(),
        }
    }
}
