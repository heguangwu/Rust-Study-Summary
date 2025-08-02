use rust_decimal::{Decimal, MathematicalOps};

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Power(Box<Node>, Box<Node>),
    Negative(Box<Node>),
    Number(Decimal)
}

impl Node {
    pub fn eval(&self) -> Decimal {
        use Node::*;
        match self {
            Add(left, right) => left.eval() + right.eval(),
            Sub(left, right) => left.eval() - right.eval(),
            Mul(left, right) => left.eval() * right.eval(),
            Div(left, right) => left.eval() / right.eval(),
            Power(left, right) => left.eval().powd( right.eval()),
            Negative(node) => -node.eval(),
            Number(decimal) => *decimal,
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use super::*;

    #[test]
    fn test_all() {
        let add: Node = Node::Add(
            Box::new(Node::Number(dec!(12))), Box::new(Node::Number(dec!(1)))
        );
        assert_eq!(add.eval(), dec!(13));

        let mul: Node = Node::Mul(
            Box::new(Node::Number(dec!(12))), Box::new(Node::Number(dec!(3)))
        );
        assert_eq!(mul.eval(), dec!(36));
    }
}
