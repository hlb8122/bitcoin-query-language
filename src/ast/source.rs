pub enum Source {
    Transaction,
    Hash,
    Input(InputSlice, usize),
    Output(OutputSlice, usize),
}

pub enum InputSlice {
    All,
    Script,
    Sequence,
}

pub enum OutputSlice {
    All,
    Script,
    Value,
}
