use crate::{fitch::Fitch, ui::Renderer};

struct App {
    model: Fitch,
    renderer: Renderer,
    state: State,
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
