use crate::calc::{ast::Node, error::{CalcError, CalcResult}, token::{OperatorPrecedence, Token}, tokenizer::Tokenizer};


pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token
}

impl <'a> Parser<'a> {
    pub fn new(expression:&'a str) -> CalcResult<Self> {
        let mut tokenizer = Tokenizer::new(expression);
        let current_token = tokenizer.next().ok_or_else(
            || CalcError::UnexpectedChar(tokenizer.get_unexpected_char().unwrap())
        )?;
        Ok(Parser { tokenizer: tokenizer, current_token: current_token })
    }

    pub fn parse(&mut self) -> CalcResult<Node> {
        // 初始优先级必须是最低优先级
        self.parse_expression(OperatorPrecedence::Default)
    }
}

impl <'a> Parser<'a> {
    fn parse_expression(&mut self, precedence: OperatorPrecedence) -> CalcResult<Node> {
        //解析表达式起始，应该为数字或带符号的数字或括号包起来的表达式
        let mut expr = self.parse_number_or_expr()?;
        println!("expr: {:?}, {:?}", expr, self.current_token);

        //这里从树的角度解析，其实就是一直找到子节点，如果下一个优先级小于当前优先级，表示为父节点，当前子树遍历完成
        //如 1 * 2 + 4 * 5: 因为 + 的优先级低于 *， 所以先创建子树 1 * 2，后面乘法优先级高，创建子树 4 * 5，最后才是 + 
        println!("======{:?}, {:?}", self.current_token.get_precedence(), precedence);
        while self.current_token.get_precedence() > precedence {
            println!("precedence==={:?}, {:?}", self.current_token.get_precedence(), precedence);
            expr = self.parse_binary_expression(expr)?;
            println!("-111--expr----> {:?}", expr);
        }
        println!("---expr----> {:?}", expr);
        Ok(expr)
    }

    fn parse_binary_expression(&mut self, left_expr: Node) -> CalcResult<Node> {
        match self.current_token {
            Token::Add => {
                self.next_token()?;
                //递归解析下一个表达式
                println!("ADD: {:?}, {:?}", left_expr, self.current_token);
                let right_expr = self.parse_expression(OperatorPrecedence::AddOrSub)?;
                println!("**ADD***left:{:?}, right:{:?}", left_expr, right_expr);
                Ok(Node::Add(Box::new(left_expr), Box::new(right_expr)))
            },
            Token::Sub => {
                self.next_token()?;
                //递归解析下一个表达式
                println!("SUB: {:?}, {:?}", left_expr, self.current_token);
                let right_expr = self.parse_expression(OperatorPrecedence::AddOrSub)?;
                println!("***SUB**left:{:?}, right:{:?}", left_expr, right_expr);
                Ok(Node::Sub(Box::new(left_expr), Box::new(right_expr)))
            },
            Token::Mul => {
                self.next_token()?;
                //递归解析下一个表达式
                println!("MUL: {:?}, {:?}", left_expr, self.current_token);
                let right_expr = self.parse_expression(OperatorPrecedence::MulOrDiv)?;
                println!("***MUL**left:{:?}, right:{:?}", left_expr, right_expr);
                Ok(Node::Mul(Box::new(left_expr), Box::new(right_expr)))
            },
            Token::Div => {
                self.next_token()?;
                //递归解析下一个表达式
                let right_expr = self.parse_expression(OperatorPrecedence::MulOrDiv)?;
                println!("**Div***left:{:?}, right:{:?}", left_expr, right_expr);
                Ok(Node::Div(Box::new(left_expr), Box::new(right_expr)))
            },
            Token::Caret => {
                self.next_token()?;
                //递归解析下一个表达式
                let right_expr = self.parse_expression(OperatorPrecedence::Power)?;
                println!("*****left:{:?}, right:{:?}", left_expr, right_expr);
                Ok(Node::Power(Box::new(left_expr), Box::new(right_expr)))
            },
            _ => unreachable!(),
        }
    }

    //将下一个token保存到
    fn next_token(&mut self) -> CalcResult<()> {
        self.current_token = self.tokenizer.next().ok_or_else(||{
            CalcError::UnexpectedChar(self.tokenizer.get_unexpected_char().unwrap())
        })?;
        Ok(())
    }

    //解析数字有如下几种可能：1）数字；2）负号；3）括号表达式
    fn parse_number_or_expr(&mut self) -> CalcResult<Node> {
        match self.current_token {
            Token::Number(n) => {
                self.next_token()?;
                Ok(Node::Number(n))
            }
            Token::Sub => { //这里判断的是负号
                self.next_token()?;
                let expr = self.parse_expression(OperatorPrecedence::Negative)?;
                Ok(Node::Negative(Box::new(expr)))
            },
            Token::LeftParen => {
                self.next_token()?;
                //括号内的表达式
                let expr = self.parse_expression(OperatorPrecedence::Default)?;
                if self.current_token != Token::RightParen {
                    if self.current_token == Token::EOF {
                        return Err(CalcError::InvalidOperator(String::from("不完整的表达式")));
                    }
                    return Err(CalcError::InvalidOperator(format!("期望右括号，但是遇到：{}", self.current_token)));
                }
                //跳过右边括号
                self.next_token()?;
                Ok(expr)
            },
            _ => {
                if self.current_token == Token::EOF {
                    return Err(CalcError::InvalidOperator(String::from("不完整的表达式")));
                }
                Err(CalcError::InvalidOperator(format!("期望数字或表达式，但是遇到：{}", self.current_token)))
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use super::*;


    #[test]
    fn test_parser() {
        let mut parser = Parser::new("1 + 2 * 3 -4").unwrap();
        assert_eq!(
            parser.parse().unwrap(),
            Node::Sub(
                Box::new(Node::Add(
                    Box::new(Node::Number(dec!(1))), 
                    Box::new(Node::Mul(Box::new(Node::Number(dec!(2))), Box::new(Node::Number(dec!(3)))))),
                ),
                Box::new(Node::Number(dec!(4)))
            )
        );
    }
}