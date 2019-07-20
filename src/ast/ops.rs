pub mod bool_op {
    pub enum Binary {
        And,
        Or,
    }

    pub enum Unary {
        Neg,
    }
}

pub mod byte_op {
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
}

pub enum Relation {
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Equal,
    NotEqual,
}
