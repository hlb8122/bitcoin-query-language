mod mask;
mod ops;
mod source;

use bitcoin::{consensus::encode::Encodable, util::psbt::serialize::Serialize, Transaction};
use num_bigint::BigUint;

pub use mask::Mask;
pub use ops::*;
pub use source::*;

pub enum Transform {
    Raw,
    Mask(Mask),
    Len,
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

pub fn resolve_predicate(predicate: Predicate, tx: &Transaction) -> bool {
    match predicate {
        Predicate::Constant(x) => x,
        Predicate::Unary(bool_op::Unary::Neg, x) => !resolve_predicate(*x, tx),
        Predicate::Binary(a, op, b) => match op {
            bool_op::Binary::And => resolve_predicate(*a, tx) && resolve_predicate(*b, tx),
            bool_op::Binary::Or => resolve_predicate(*a, tx) || resolve_predicate(*b, tx),
        },
        Predicate::Constraint(bytes_a, rel, bytes_b) => {
            let resolved_a = resolve_bytes(bytes_a, tx);
            let resolved_b = resolve_bytes(bytes_b, tx);
            match rel {
                Relation::Equal => resolved_a == resolved_b,
                Relation::NotEqual => resolved_a != resolved_b,
                Relation::GreaterThan => resolved_a > resolved_b,
                Relation::GreaterThanEq => resolved_a >= resolved_b,
                Relation::LessThan => resolved_a < resolved_b,
                Relation::LessThanEq => resolved_a <= resolved_b,
            }
        }
    }
}

fn resolve_bytes(bytes: Bytes, tx: &Transaction) -> Vec<u8> {
    match bytes {
        Bytes::Raw(x) => x,
        Bytes::Unary(op, x) => match op {
            byte_op::Unary::Flip => resolve_bytes(*x, tx).into_iter().map(|b| b ^ 0).collect(),
            byte_op::Unary::Reverse => {
                let mut raw = resolve_bytes(*x, tx);
                raw.reverse();
                raw
            }
        },
        Bytes::Binary(x, op, y) => {
            let raw_x = resolve_bytes(*x, tx);
            let raw_y = resolve_bytes(*y, tx);
            match op {
                byte_op::Binary::Xor => {
                    let len_x = raw_x.len();
                    let len_y = raw_y.len();
                    let (len_correct, mut new, other) = if len_x < len_y {
                        (len_x, raw_y, raw_x)
                    } else {
                        (len_y, raw_x, raw_y)
                    };

                    for i in 0..len_correct {
                        *new.get_mut(i).unwrap() ^= other.get(i).unwrap();
                    }
                    new
                }
                byte_op::Binary::And => {
                    let len_x = raw_x.len();
                    let len_y = raw_y.len();
                    let (len_correct, mut new, other) = if len_x < len_y {
                        (len_x, raw_y, raw_x)
                    } else {
                        (len_y, raw_x, raw_y)
                    };

                    for i in 0..len_correct {
                        *new.get_mut(i).unwrap() &= other.get(i).unwrap();
                    }
                    new
                }
                byte_op::Binary::Or => {
                    let len_x = raw_x.len();
                    let len_y = raw_y.len();
                    let (len_correct, mut new, other) = if len_x < len_y {
                        (len_x, raw_y, raw_x)
                    } else {
                        (len_y, raw_x, raw_y)
                    };

                    for i in 0..len_correct {
                        *new.get_mut(i).unwrap() |= other.get(i).unwrap();
                    }
                    new
                }
                byte_op::Binary::Add => {
                    let big_x = BigUint::from_bytes_be(&raw_x);
                    let big_y = BigUint::from_bytes_be(&raw_y);
                    (big_x + big_y).to_bytes_be()
                }
                byte_op::Binary::Subtract => {
                    let big_x = BigUint::from_bytes_be(&raw_x);
                    let big_y = BigUint::from_bytes_be(&raw_y);
                    (big_x - big_y).to_bytes_be()
                }
                byte_op::Binary::Multiply => {
                    let big_x = BigUint::from_bytes_be(&raw_x);
                    let big_y = BigUint::from_bytes_be(&raw_y);
                    (big_x * big_y).to_bytes_be()
                }
            }
        }
        Bytes::Sourced(source, transform) => {
            let bytes = match source {
                Source::Transaction => tx.serialize(),
                Source::Hash => tx.txid()[..].to_vec(),
                Source::Input(slice, index) => {
                    let input = match tx.input.get(index) {
                        Some(some) => some,
                        None => return vec![],
                    };
                    match slice {
                        InputSlice::All => {
                            let mut raw = Vec::new();
                            input.consensus_encode(&mut raw).unwrap();
                            raw
                        }
                        InputSlice::Script => input.script_sig.to_bytes(),
                        InputSlice::Sequence => input.sequence.to_be_bytes().to_vec(),
                    }
                }
                Source::Output(slice, index) => {
                    let output = match tx.output.get(index) {
                        Some(some) => some,
                        None => return vec![],
                    };
                    match slice {
                        OutputSlice::All => {
                            let mut raw = Vec::new();
                            output.consensus_encode(&mut raw).unwrap();
                            raw
                        }
                        OutputSlice::Script => output.script_pubkey.to_bytes(),
                        OutputSlice::Value => output.value.to_be_bytes().to_vec(),
                    }
                }
            };

            match transform {
                Transform::Raw => bytes,
                Transform::Mask(mask) => mask.apply(bytes),
                Transform::Len => bytes.len().to_be_bytes().to_vec(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{consensus::encode::deserialize, util::misc::hex_bytes, Transaction};

    fn generate_tx() -> Transaction {
        let hex_tx = hex_bytes("0100000001a15d57094aa7a21a28cb20b59aab8fc7d1149a3bdbcddba9c622e4f5f6a99ece010000006c493046022100f93bb0e7d8db7bd46e40132d1f8242026e045f03a0efe71bbb8e3f475e970d790221009337cd7f1f929f00cc6ff01f03729b069a7c21b59b1736ddfee5db5946c5da8c0121033b9b137ee87d5a812d6f506efdd37f0affa7ffc310711c06c7f3e097c9447c52ffffffff0100e1f505000000001976a9140389035a9225b3839e2bbf32d826a1e222031fd888ac00000000").unwrap();
        deserialize(&hex_tx).unwrap()
    }

    #[test]
    fn test_relations() {
        let dummy = generate_tx();

        // Equality
        let pred = Predicate::Constraint(
            Bytes::Raw(vec![0, 1, 2]),
            Relation::Equal,
            Bytes::Raw(vec![0, 1, 2]),
        );
        assert!(resolve_predicate(pred, &dummy));

        let pred = Predicate::Constraint(
            Bytes::Raw(vec![0]),
            Relation::Equal,
            Bytes::Raw(vec![0, 1, 2]),
        );
        assert!(!resolve_predicate(pred, &dummy));

        // Less than
        let pred = Predicate::Constraint(
            Bytes::Raw(vec![0]),
            Relation::LessThan,
            Bytes::Raw(vec![0, 1]),
        );
        assert!(resolve_predicate(pred, &dummy));

        let pred = Predicate::Constraint(
            Bytes::Raw(vec![1, 0]),
            Relation::LessThan,
            Bytes::Raw(vec![0, 1]),
        );
        assert!(!resolve_predicate(pred, &dummy));

        // Greater than
        let pred = Predicate::Constraint(
            Bytes::Raw(vec![5, 0]),
            Relation::GreaterThan,
            Bytes::Raw(vec![4, 200]),
        );
        assert!(resolve_predicate(pred, &dummy));

        let pred = Predicate::Constraint(
            Bytes::Raw(vec![0]),
            Relation::GreaterThan,
            Bytes::Raw(vec![0, 1]),
        );
        assert!(!resolve_predicate(pred, &dummy));

        // Not equal
        let pred = Predicate::Constraint(
            Bytes::Raw(vec![5, 0]),
            Relation::NotEqual,
            Bytes::Raw(vec![4, 3]),
        );
        assert!(resolve_predicate(pred, &dummy));

        let pred = Predicate::Constraint(
            Bytes::Raw(vec![0, 1]),
            Relation::GreaterThan,
            Bytes::Raw(vec![0, 1]),
        );
        assert!(!resolve_predicate(pred, &dummy));
    }

    #[test]
    fn test_bool_op() {
        let dummy = generate_tx();

        // Or
        let pred_or = Predicate::Binary(
            Box::new(Predicate::Constant(true)),
            bool_op::Binary::Or,
            Box::new(Predicate::Constant(false)),
        );
        assert!(resolve_predicate(pred_or, &dummy));

        let pred_or = Predicate::Binary(
            Box::new(Predicate::Constant(false)),
            bool_op::Binary::Or,
            Box::new(Predicate::Constant(false)),
        );
        assert!(!resolve_predicate(pred_or, &dummy));

        // And
        let pred_and = Predicate::Binary(
            Box::new(Predicate::Constant(true)),
            bool_op::Binary::Or,
            Box::new(Predicate::Constant(true)),
        );
        assert!(resolve_predicate(pred_and, &dummy));

        let pred_and = Predicate::Binary(
            Box::new(Predicate::Constant(true)),
            bool_op::Binary::And,
            Box::new(Predicate::Constant(false)),
        );
        assert!(!resolve_predicate(pred_and, &dummy));

        // Contrapositive
        let pred_and = Predicate::Unary(
            bool_op::Unary::Neg,
            Box::new(Predicate::Constant(false)),
        );
        assert!(resolve_predicate(pred_and, &dummy));

        let pred_and = Predicate::Unary(
            bool_op::Unary::Neg,
            Box::new(Predicate::Constant(true)),
        );
        assert!(!resolve_predicate(pred_and, &dummy));
    }
}
