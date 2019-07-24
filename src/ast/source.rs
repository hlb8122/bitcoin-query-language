use super::*;

pub enum InputSlice {
    All,
    Script,
    Sequence,
}

fn parse_in_slice<'a>(i: &'a str) -> IResult<&'a str, InputSlice, VerboseError<&'a str>> {
    alt((
        map(tag("all"), |_| InputSlice::All),
        map(tag("script"), |_| InputSlice::Script),
        map(tag("seq"), |_| InputSlice::Sequence),
    ))(i)
}

pub enum OutputSlice {
    All,
    Script,
    Value,
}

fn parse_out_slice<'a>(i: &'a str) -> IResult<&'a str, OutputSlice, VerboseError<&'a str>> {
    alt((
        map(tag("all"), |_| OutputSlice::All),
        map(tag("script"), |_| OutputSlice::Script),
        map(tag("value"), |_| OutputSlice::Value),
    ))(i)
}

pub enum Source {
    Transaction,
    Hash,
    Input(InputSlice, usize),
    Output(OutputSlice, usize),
}

fn parse_source<'a>(i: &'a str) -> IResult<&'a str, Source, VerboseError<&'a str>> {
    alt((
        map(tag("tx"), |_| Source::Transaction),
        map(tag("hash"), |_| Source::Hash),
    ))(i)
}
