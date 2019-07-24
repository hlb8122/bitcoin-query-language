mod mask;
mod ops;
mod resolution;
mod source;

use bitcoin::{consensus::encode::Encodable, util::psbt::serialize::Serialize, Transaction};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, digit1, multispace0, multispace1, one_of},
    combinator::{cut, map, map_res, opt},
    error::{context, VerboseError},
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use num_bigint::BigUint;

pub use mask::Mask;
pub use ops::*;
pub use resolution::*;
pub use source::*;

pub enum Transform {
    Raw,
    Mask(Mask),
    Len,
}

fn parse_transform<'a>(i: &'a str) -> IResult<&'a str, Transform, VerboseError<&'a str>> {
    map(
        context("keyword", preceded(tag(":"), cut(alpha1))),
        |sym_str: &str| match sym_str {
            "raw" => Transform::Raw,
            "len" => Transform::Len,
            _ => unreachable!(),
        },
    )(i)
}

pub enum Bytes {
    Sourced(Source, Transform),
    Raw(Vec<u8>),
    Unary(byte_op::Unary, Box<Bytes>),
    Binary(Box<Bytes>, byte_op::Binary, Box<Bytes>),
}

pub enum Predicate {
    Unary(bool_op::Unary, Box<Predicate>),
    Binary(Box<Predicate>, bool_op::Binary, Box<Predicate>),
    Constraint(Bytes, Relation, Bytes),
    Constant(bool),
}
