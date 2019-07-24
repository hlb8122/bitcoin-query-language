use super::*;

pub mod bool_op {
    use super::*;
    pub enum Binary {
        And,
        Or,
    }

    fn parse_binary<'a>(i: &'a str) -> IResult<&'a str, Binary, VerboseError<&'a str>> {
        let (i, t) = one_of("&|")(i)?;

        Ok((
            i,
            match t {
                '&' => Binary::And,
                '|' => Binary::Or,
                _ => unreachable!(),
            },
        ))
    }

    pub enum Unary {
        Not,
    }

    fn parse_unary<'a>(i: &'a str) -> IResult<&'a str, Unary, VerboseError<&'a str>> {
        let (i, t) = one_of("!")(i)?;

        Ok((
            i,
            match t {
                '!' => Unary::Not,
                _ => unreachable!(),
            },
        ))
    }
}

pub mod byte_op {
    use super::*;
    pub enum Unary {
        Flip,
        Reverse,
    }

    pub enum Binary {
        Add,
        Subtract,
        Multiply,
        Xor,
        And,
        Or,
    }

    fn parse_binary<'a>(i: &'a str) -> IResult<&'a str, Binary, VerboseError<&'a str>> {
        let (i, t) = one_of("+-*^&|")(i)?;

        Ok((
            i,
            match t {
                '+' => Binary::Add,
                '-' => Binary::Subtract,
                '*' => Binary::Multiply,
                '^' => Binary::Xor,
                '&' => Binary::And,
                '|' => Binary::Or,
                _ => unreachable!(),
            },
        ))
    }
}

pub enum Relation {
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Equal,
    NotEqual,
}

fn parse_relation_strict<'a>(i: &'a str) -> IResult<&'a str, Relation, VerboseError<&'a str>> {
    let (i, t) = one_of("<>=")(i)?;

    Ok((
        i,
        match t {
            '<' => Relation::LessThan,
            '>' => Relation::GreaterThan,
            '=' => Relation::Equal,
            _ => unreachable!(),
        },
    ))
}

fn parse_relation<'a>(i: &'a str) -> IResult<&'a str, Relation, VerboseError<&'a str>> {
    alt((
        parse_relation_strict,
        alt((
            map(tag("<="), |_| Relation::LessThanEq),
            map(tag(">="), |_| Relation::GreaterThanEq),
        )),
    ))(i)
}
