use std::{ iter::Peekable, str::Chars};
use rust_decimal::Decimal;

use crate::calc::token::Token;

pub struct Tokenizer<'a> {
    expression: Peekable<Chars<'a>>,
    reached_end: bool,
    unexpected_char: Option<char>,
}

impl <'a> Tokenizer<'a> {
    pub fn new(expression: &'a str) -> Self {
        Self {
            expression: expression.chars().peekable(),
            reached_end: false,
            unexpected_char: None
        }
    }

    pub fn get_unexpected_char(&self) -> Option<char> {
        self.unexpected_char
    }

    pub fn reached_end(&self) -> bool {
        self.reached_end
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reached_end {
            return None;
        }
        let next_char = self.expression.next();
        match next_char {
            Some(c) if c.is_numeric() => {
                let mut number = String::from(c);
                // 尽可能读取更多的数字
                while let Some(n) = self.expression.next_if(|ch| ch.is_numeric() || *ch == '.') {
                    number.push(n);
                }
                let num: Result<Decimal, _> = number.parse();
                match num {
                    Ok(n) => Some(Token::Number(n)),
                    Err(_) => {
                        self.unexpected_char = Some('.');
                        self.reached_end = true;
                        None
                    }
                }
            },
            Some(chr) if chr.is_whitespace() => {
                //跳过所有的空白字符，因为空白字符不需要返回，所以这里要递归调用自己直至返回
                while let Some(_) = self.expression.next_if(|c| c.is_whitespace()) {}
                self.next()
            },
            Some('+') => Some(Token::Add),
            Some('-') => Some(Token::Sub),
            Some('*') => Some(Token::Mul),
            Some('/') => Some(Token::Div),
            Some('^') => Some(Token::Caret),
            Some('(') => Some(Token::LeftParen),
            Some(')') => Some(Token::RightParen),
            Some(_) => {
                self.unexpected_char = next_char;
                self.reached_end = true;
                None
            },
            None => {
                self.reached_end = true;
                Some(Token::EOF)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::dec;

    #[test]
    fn test_add_sub() {
        let tokenizer = Tokenizer::new(" 1+ 2   - 4 ");
        assert_eq!(
            tokenizer.collect::<Vec<Token>>(),
            vec![
                Token::Number(dec!(1)),
                Token::Add,
                Token::Number(dec!(2)),
                Token::Sub,
                Token::Number(dec!(4)),
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_all() {
        let tokenizer = Tokenizer::new(" (1 + 2 ^ 5) / 9 *12.4 ");
        assert_eq!(
            tokenizer.collect::<Vec<Token>>(),
            vec![
                Token::LeftParen,
                Token::Number(dec!(1)),
                Token::Add,
                Token::Number(dec!(2)),
                Token::Caret,
                Token::Number(dec!(5)),
                Token::RightParen,
                Token::Div,
                Token::Number(dec!(9)),
                Token::Mul,
                Token::Number(dec!(12.4)),
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_number_error() {
        let tokenizer = Tokenizer::new(" 1+ 2.3.4");
        assert_eq!(
            tokenizer.collect::<Vec<Token>>(),
            vec![
                Token::Number(dec!(1)),
                Token::Add,
            ]
        );
    }

    #[test]
    fn test_number_multi_zero() {
        let tokenizer = Tokenizer::new(" 1+ 0000.4");
        assert_eq!(
            tokenizer.collect::<Vec<Token>>(),
            vec![
                Token::Number(dec!(1)),
                Token::Add,
                Token::Number(dec!(0.4)),
                Token::EOF
            ]
        );
    }
}
