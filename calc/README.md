# 通过手动实现计算器来理解编译器是如何工作的

> 为简化程序，本计算器只实现加、减、乘、除和幂次操作，且只支持小括号。
> 本文是学习B站[冷笑浅兮的视频](https://www.bilibili.com/video/BV1R4VpzgESS?spm_id_from=333.788.videopod.sections&vd_source=151470303bff86fb1580c8c795b22fc1)的一个总结，想看视频学习的请到链接地址。

一般来说，计算器包含如下三个步骤：

1. 分词：将输入文本分成一个个`token`
2. 解析：将token建构为抽象语法树（`AST`）
3. 评估：执行抽象语法树得到结果

## 分词

在分词之前要定义分词转化后的数据结构，包括：

- 运算符：+、-、*、/、^
- 小括号: (、)
- 数字

```rust
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token {
    Add,
    Sub,
    Mul,
    Div,
    Caret,
    LeftParen,
    RightParen,
    Number(Decimal),
    EOF
}
```

以及运算符(`+ - * / ^`)的优先级定义：

```rust
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum OperatorPrecedence {
    Default,
    AddOrSub,
    MulOrDiv,
    Power,
    Negative
}
```

分词实际上就是依次读取一个字符将其转换为对应的`Token`的过程。比如读取到`+`返回`Token::Add`，唯一有区别的就是值，在读取值的时候，需要窥探下一个字符是否满足条件，将满足条件的字符最终一起转换为值。

```rust
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
```

## 解析

解析`AST`的过程就是递归全部数据建构树的过程。代码如下：

```rust
pub fn parse(&mut self) -> CalcResult<Node> {
    self.parse_expression(OperatorPrecedence::Default)
}

fn parse_expression(&mut self, precedence: OperatorPrecedence) -> CalcResult<Node> {
    let mut expr = self.parse_number_or_expr()?;
    while self.current_token.get_precedence() > precedence {
        expr = self.parse_binary_expression(expr)?;
    }
    Ok(expr)
}

fn parse_binary_expression(&mut self, left_expr: Node) -> CalcResult<Node> {
    match self.current_token {
        Token::Add => {
            self.next_token()?;
            let right_expr = self.parse_expression(OperatorPrecedence::AddOrSub)?;
            Ok(Node::Add(Box::new(left_expr), Box::new(right_expr)))
        },
        Token::Sub => {
            self.next_token()?;
            let right_expr = self.parse_expression(OperatorPrecedence::AddOrSub)?;
            Ok(Node::Sub(Box::new(left_expr), Box::new(right_expr)))
        },
        Token::Mul => {
            self.next_token()?;
            let right_expr = self.parse_expression(OperatorPrecedence::MulOrDiv)?;
            Ok(Node::Mul(Box::new(left_expr), Box::new(right_expr)))
        },
        Token::Div => {
            self.next_token()?;
            let right_expr = self.parse_expression(OperatorPrecedence::MulOrDiv)?;
            Ok(Node::Div(Box::new(left_expr), Box::new(right_expr)))
        },
        Token::Caret => {
            self.next_token()?;
            let right_expr = self.parse_expression(OperatorPrecedence::Power)?;
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
```

上述代码看起来有点抽象难懂，这里举个例子说明一下，以`1 + 2 * 3 - 4`为例：

1. `parse_expression`函数中得到第一个`token`是数字`1`，获取下一个设置`current_token`是`+`
2. 因为`+`优先级大于默认值，所以进入`while`循环，在`parse_binary_expression`函数中进入`Token::Add`分支
3. 获取下一个设置`current_token`是数字`2`，递归进入`parse_expression`函数，先解析`current_token`得到数字`2`，获取并设置`current_token`是`*`
4. 因为`*`优先级大于`+`，进入`while`循环，在`parse_binary_expression`函数中进入`Token::Mul`分支
5. 获取下一个设置`current_token`是数字`3`，递归进入`parse_expression`函数，先解析`current_token`得到数字`3`，获取并设置`current_token`是`-`
6. 因为`-`优先级小于`*`，直接返回`expr`，递归返回`Mul(Number(2), Number(3))`
7. 再递归上层返回`Add(Number(1), Mul(Number(2), Number(3)))`
8. 此时，`current_token`是`-`，`precedence`是`Default`，满足条件，进入`parse_binary_expression`函数进入`Token::Sub`分支
9. 递归进入`parse_expression`函数，先解析`current_token`得到数字`4`，获取并设置`current_token`是`EOF`，此时不满足循环条件，递归返回

看了上述过程后，再来讲一下这个递归的设计：

- 初始传入的运算符的优先级必须是低于所有正常运算符的优先级，以便递归能进入
- 终止符 `EOF` 是递归完成标志，其优先级要能让递归结束
- 每次解析当前`Token`数字后，必须要将`current_token`设置为下一个`Token`，此时`current_token`肯定是运算符，当前运算符优先级低于传入的优先级，这个递归就结束

## 评估

评估实现比较简单，其实就是根据运算符计算即可。
