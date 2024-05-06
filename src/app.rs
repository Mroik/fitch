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
        Ok(App {
            model: Fitch::new(),
            renderer: Renderer::new()?,
            state: State::Noraml,
            expression_buffer: String::new(),
        })
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
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    _ => todo!(),
                }
            }
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
            ]
            .join("   ")
            .to_string(),
            _ => "".to_string(),
        }
    }
}

enum State {
    Noraml,
    AbsurdumState(AbsurdumState),
    AndState(AndState),
    OrState(OrState),
    NotState(NotState),
    ImpliesState(ImpliesState),
    IffState(IffState),
}

enum AbsurdumState {
    IntroduceGetAssumption1,
    IntroduceGetAssumption2(usize),
}

enum AndState {
    IntroduceGetLeftAssumption,
    IntroduceGetRightAssumption(usize),
    EliminateGetAssumption,
    EliminateGetProposition(usize),
}

enum OrState {
    IntroduceGetAssumption,
    IntroduceGetProposition(usize),
    EliminateGetAssumption,
    EliminateGetLeftSubproof(usize),
    EliminateGetRightSubproof(usize, usize),
}

enum NotState {
    Introduce,
    Eliminate,
}

enum ImpliesState {
    Introduce,
    EliminateGetAssumption,
    EliminateGetLeft(usize),
}

enum IffState {
    IntroduceGetLeftSubproof,
    IntroduceGetRightSubproof(usize),
    EliminateGetAssumption,
    EliminateGetTruth(usize),
}
