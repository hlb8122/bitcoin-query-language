use super::*;

pub struct Mask(Vec<bool>);

impl Mask {
    pub fn invert(&mut self) -> Self {
        Mask(self.0.iter().map(|b| !b).collect())
    }

    pub fn apply(&self, other: Vec<u8>) -> Vec<u8> {
        other
            .iter()
            .enumerate()
            .filter_map(|(i, byte)| match self.0.get(i) {
                Some(true) => Some(*byte),
                _ => None,
            })
            .collect()
    }
}

fn parse_bool<'a>(i: &'a str) -> IResult<&'a str, bool, VerboseError<&'a str>> {
    let (i, t) = one_of("01")(i)?;

    Ok((
        i,
        match t {
            '1' => true,
            '0' => false,
            _ => unreachable!(),
        },
    ))
}

fn parse_mask<'a>(i: &'a str) -> IResult<&'a str, Mask, VerboseError<&'a str>> {
    let mut v = i;
    let res_vec: Result<Vec<bool>, _> = std::iter::from_fn(move || match parse_bool(v) {
        Ok((i, o)) => {
            v = i;
            Some(Ok(o))
        }
        Err(e) => return Some(Err(e)),
    })
    .collect();
    Ok(("", Mask(res_vec?)))
}
