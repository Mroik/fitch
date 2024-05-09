#[derive(PartialEq)]
pub enum State {
    Noraml,
    IntroduceChoice,
    EliminateChoice,
    AddAssumption,
    AddSubproof,
    Reiterate,
    AbsurdumState(AbsurdumState),
    AndState(AndState),
    OrState(OrState),
    NotState(NotState),
    ImpliesState(ImpliesState),
    IffState(IffState),
    Quit,
}

#[derive(PartialEq)]
pub enum AbsurdumState {
    IntroduceGetAssumption1,
    IntroduceGetAssumption2(usize),
    EliminateGetAssumption,
    EliminateGetProposition(usize),
}

#[derive(PartialEq)]
pub enum AndState {
    IntroduceGetLeftAssumption,
    IntroduceGetRightAssumption(usize),
    EliminateGetAssumption,
    EliminateGetProposition(usize),
}

#[derive(PartialEq)]
pub enum OrState {
    IntroduceGetAssumption,
    IntroduceGetProposition(usize),
    EliminateGetAssumption,
    EliminateGetLeftSubproof(usize),
    EliminateGetRightSubproof(usize, usize),
}

#[derive(PartialEq)]
pub enum NotState {
    Introduce,
    Eliminate,
}

#[derive(PartialEq)]
pub enum ImpliesState {
    Introduce,
    EliminateGetAssumption,
    EliminateGetLeft(usize),
}

#[derive(PartialEq)]
pub enum IffState {
    IntroduceGetLeftSubproof,
    IntroduceGetRightSubproof(usize),
    EliminateGetAssumption,
    EliminateGetTruth(usize),
}
