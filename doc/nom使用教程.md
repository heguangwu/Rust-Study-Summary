# `nom` 使用教程


![nom](./img/nom.png "nom")

<details>
<summary><b>目录</b></summary>

- [`nom` 使用教程](#nom-使用教程)
  - [基本概念](#基本概念)
  - [解析器和组合器](#解析器和组合器)
    - [概述](#概述)
    - [基本元素](#基本元素)
    - [选择组合器](#选择组合器)
    - [序列组合器](#序列组合器)
    - [重复解析器](#重复解析器)
    - [数值型](#数值型)
    - [其它解析器](#其它解析器)
  - [输入数据类型](#输入数据类型)
  - [解析器测试与调试](#解析器测试与调试)
  - [错误处理](#错误处理)
    - [错误输出格式](#错误输出格式)
    - [自定义错误](#自定义错误)
  - [字符测试函数](#字符测试函数)
  - [一些例子](#一些例子)
    - [C语言注释](#c语言注释)
    - [去掉字符串前后空格](#去掉字符串前后空格)
    - [十六进制转换](#十六进制转换)
    - [实现`FromStr`](#实现fromstr)
  - [参考文献](#参考文献)

</details>

## 基本概念

`nom` 是一个解析器和组合器库，其可以在`no_std`中建构，特点是：

- 解析安全
- 流模式
- 零拷贝

[官网](https://docs.rs/nom/latest/nom/)的例子如下：

```rust
use nom::{
  IResult,
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
  sequence::tuple};

#[derive(Debug,PartialEq)]
pub struct Color {
  pub red:     u8,
  pub green:   u8,
  pub blue:    u8,
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
  map_res(
    take_while_m_n(2, 2, is_hex_digit),
    from_hex
  )(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
  let (input, _) = tag("#")(input)?;
  let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

  Ok((input, Color { red, green, blue }))
}

fn main() {
  assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
    red: 47,
    green: 20,
    blue: 223,
  })));
}
```

从上可以看出，`nom`是一个一个的`吃掉数据`。
在讲述解析器和组合器之前，先了解一下其结果返回 `IResult`，其定义如下:

```rust
pub type IResult<I, O, E = error::Error<I>> = Result<(I, O), Err<E>>;
```

`IResult`就是`Result`的一个别名，解析成功返回`Ok((I, O))`，其中`I`为解析后剩余的数据，`O`为解析得到的数据。

## 解析器和组合器

### 概述

在`nom::bytes`包下面有`streaming` 和 `complete` 两个不同类型的组合器，其区别是：

- `complete`: 用于分解已经具备完整数据，如小文件、完整网络包等，如不完整返回错误
- `streaming`: 用于分解通过流式读取的数据，如网络读取、大文件等，如不完整返回`Err::Incomplete`

限于篇幅，这里只介绍主要的解析器。

### 基本元素

- `char`: 匹配单个字符，输入`h00114432`，使用`char('h')`返回`Ok(("00114432", 'h'))`，如果第一个不是以`h`则返回错误。
- `is_a`: 匹配字符串里面的任意一个，输入`abbacbcc`，使用`is_a("ab")`返回`Ok(("cbcc", "abba"))`
- `is_not`: 不满足字符串里面的任意一个则匹配成功，是`is_a`的否操作，输入`abbacbcc`，使用`is_not("c")`返回`Ok(("cbcc", "abba"))`
- `one_of`: 第一个字符在匹配字符串，输入`abbacbcc`，使用`one_of("ab")`返回`Ok(("bbacbcc", 'a'))`
- `none_of`: 第一个字符不在匹配字符串里面，输入`aabbacbcc`，使用`none_of("bc")`返回`Ok(("abbacbc", 'a'))`
- `tag`: 匹配一组特定的字符或字节，输入`aabbacbcc`，使用`tag("aabb")`返回`Ok(("acbcc", "aabb"))`
- `tag_no_case`: 同`tag`，忽略大小写，注意对于`unicode`字符结果不确定，输入`aBChello`，使用`tag("abc")`返回`Err`，使用`tag_no_case("abc")`返回`Ok(("hello", "aBC"))`
- `take`: 获取指定数目字节的数组或字符，输入`aabbacbcc`，使用`take(4u8)`返回`Ok(("acbcc", "aabb"))`
- `take_while`: 获取直至`take_while`参数函数不满足为止，输入`hello123`，使用`take_while(is_alphabetic)`返回`Ok(("123", "hello"))`
  - `take_while1`: 和`take_while`类似，但必须要返回一个，否则返回错误，加上输入都是`123hello`，`take_while(is_alphabetic)`返回`Ok((b"123hello", []))`，而`take_while1`返回`Err`
  - `take_while_m_n`: 获取最少m个，最多n个字符满足条件的字符，否则返回错误
- `take_till`: 获取直至`take_till`参数函数满足为止，输入`123hello`，使用`take_till(is_alphabetic)`返回`Ok(("hello", "123"))`
- `take_until`: 获取直至指定字符串匹配为止，输入`123helloworld`，使用`take_until("lo")`返回`Ok(("loworld", "123hel"))`

注意，上述为简化，实际基本元素返回值的都是一个**函数**，其声明为：

```rust
impl Fn(Input) -> IResult<Input, Input, Error>）
```

需要使用这个函数再去对输入数据进行调用（或调用`parse`方法）才能得到结果。如：

```rust
let input = "hello123";
let res: IResult<&str, &str> = take(3u8)(input)?; //Ok(("lo123", "hel"))
let res: IResult<&[u8], &[u8]> = take_while_m_n(2, 2, is_alphabetic).parse(b"114432")?; //Ok(([52,52,51,50],[49,49]))
let res: IResult<&str, &str> = tag_no_case("HTTP")("Http/1.1"); //Ok(("/1.1", "Http"))
```

### 选择组合器

- `alt`: 相当于逻辑或，只要有一个满足条件，即返回该解析器的匹配结果，全部不满足返回错误
- `permutation`: 相当于逻辑与，当所有条件全部匹配时才返回各个解析器的结果，否则返回错误

使用示范如下：

```rust
let input = "hello,world";
//匹配 he
let res: IResult<&str, &str> = alt(( tag("wo"), tag("he"), take(5u8)))(input); //Ok(("llo,world", "he"))
//依次匹配，只要匹配上这些匹配上的字符就被吃掉，只有全部匹配上才返回数据，否则返回错误
let res: IResult<&str, (&str,&str,&str)> = permutation((tag("ll"), tag("he"), tag("o,")))(input); //Ok(("world", ("ll","he","o,")))
let res: IResult<&str, (&str,&str)> = permutation((tag("ab"), eof))("ab"); //Ok(("", ("ab","")))
```

### 序列组合器

- `delimited`: 依次匹配三个解析器，都满足的情况下返回第二个解析器结果，其它结果丢弃
- `preceded`: 依次匹配两个解析器，都满足的情况下返回第二个解析器的结果，第一个结果丢弃
- `terminated`: 依次匹配两个解析器，都满足的情况下返回第一个解析器的结果，第二个结果丢弃
- `pair`: 依次匹配两个解析器，都满足的情况下返回两个结果
- `separated_pair`: 依次匹配三个解析器，都满足的情况下返回第一个和第三个结果
- `tuple`: 依次匹配解析器，都满足的情况下返回全部结果，和`permutation`的区别是其必须按顺序，而`permutation`只需要匹配就可以，和顺序无关，但每一个解析器都只匹配一次

使用示范如下：

```rust
//获取括弧内的数据
let res: IResult<&str, &str> = delimited(char('('), is_not(")"), char(')')))("(hello)world");  //Ok(("world", "hello"))
//去掉"he_"前缀，获取其余内容
let res: IResult<&str, &str> = preceded(tag("he_"), rest))("he_world.txt");  //Ok(("", "world.txt"))
//去掉".txt"后缀名
let res: IResult<&str, &str> = terminated(is_not("."), tag(".txt")))("input.txt");  //Ok(("", "input"))
//获取文件名和后缀名(带".")
let res: IResult<&str, (&str, &str)> = pair(is_not("."), rest))("input.txt");  //Ok(("", ("input", ".txt")))
//获取文件名和后缀名(不带".")
let res: IResult<&str, (&str, &str)> = separated_pair(is_not("."), is_a("."), rest))("input.txt");  //Ok(("", ("input", "txt")))
//获取文件名、"."和后缀名(不带".")
let res: IResult<&str, (&str, &str, &str)> = tuple((is_not("."), is_a("."), rest)))("input.txt");  //Ok(("", ("input", ".", "txt")))
```

### 重复解析器

- `count`: 应用子解析器指定次数，有一次失败返回错误
- `many0`: 应用子解析器0次或多次，返回`Vec`
- `many1`: 同`many0`，但要求必须至少匹配成功一次，否则返回错误
- `many0_count`: `count`和`many0`的结合，返回成功匹配的次数
- `many1_count`: 同`many0_count`，但要求必须至少匹配成功一次，否则返回错误
- `many_m_n`: 同`many0`，但要求匹配最少m次，最多n次
- `many_till`: 重复匹配第一个子解析器，一旦不匹配则匹配第二解析器，如第二个不匹配则返回错误，匹配则返回两个解析器匹配的结果
- `separated_list0`: 按第一个解析器分割第二个解析器匹配，返回结果是匹配第二个解析的结果
- `separated_list1`: 同`separated_list0`，但要求至少匹配一个
- `fold_many0`: 使用第一个解析器进行分解，第二个参数是初始值，第三个参数是对第一个解析器结果进行处理，返回的是第三个函数的处理结果
- `fold_many1`: 同`fold_many0`，区别是第一个解析器至少匹配一个
- `fold_many_m_n`: 同`fold_many0`，区别是第一个解析器至少匹配m个，最多匹配n个
- `length_count`: 使用第一个解析器获取数字作为执行次数，执行第二个解析器对应次数

使用示范如下：

```rust
let res: IResult<&str, Vec<&str>> = count(take(2u8), 3)("helloworld"); //Ok(("orld", ["he","ll","ow"]]))
let res: IResult<&str, Vec<&str>> = many0(tag("ab"))("ababc"); //Ok(("c", ["ab","ab"]]))
let res: IResult<&str, usize> = many0_count(tag("ab"))("ababc"); //Ok(("c", 2]))
let res: IResult<&str, Vec<&str>> = many_m_n(1, 3, tag("ab"))("ababababc"); //Ok(("abc", ["ab","ab","ab"]]))
let res: IResult<&str, (Vec<&str>, &str)> = many_till(tag("ab"), tag("df"))("ababdfabc"); //Ok(("abc", ["ab","ab"],"df"]))
let res: IResult<&str, Vec<&str>> = separated_list0(tag(","), tag("ab"))("ab,ab,ab.ab"); //Ok((".ab", ["ab","ab","ab"]]))
let res: IResult<&[u8], u8> = fold_many0(be_u8, ||0, |acc, x| acc + x)(&[1u8,2u8,3u8]); //Ok(("[]", 6]))
let res: IResult<&str, usize> = fold_many0(tag("ab,"), ||0, |acc, x: &str| acc + x.len())("ab,ab,ab,ab"); //Ok(("ab", 9]))
let res: IResult<&[u8], Vec<&u8>> = length_count(complete::u8, take(2u8))(b"\x03ababcdef"); //Ok(([101,102], [[97,98],[97,98],[99,100]]]))
```

### 数值型

将`byte`数组按大小端顺序解析为数据的方法，包括：

- 配置大小端的(同样位于 `nom::number::complete`、`nom::number::streaming`包下)： `i16`, `i32`, `i64`, `u16`, `u32`, `u64`
- 固定大小端的
  - 大端有符号：`be_i8, be_i16, be_i24, be_i32, be_i64, be_i128`
  - 小端有符号：`le_i8, le_i16, le_i24, le_i32, le_i64, le_i128`
  - 大端无符号：`be_u8, be_u16, be_u24, be_u32, be_u64, be_u128`
  - 小端无符号：`le_u8, le_u16, le_u24, le_u32, le_u64, le_u128`
  - 大端浮点数：`be_f32, be_f64`
  - 小端浮点数：`le_f32, le_f64`
  - 浮点数解析：`float, double`
  - 十六进制解析：`hex_u32`

使用示范如下：

```rust
let res: IResult<&[u8],i16> = i16(nom::number::Endianness::Big)(b"\x10\x3f");  //Ok(([], 4159))
let res: IResult<&[u8],i16> = i16(nom::number::Endianness::Little)(b"\x10\x3f"); //Ok(([], 16144))
let res: IResult<&[u8],f32> = be_f32(b"\x1f\x2f\xf0\x3f"); //Ok(([], 3.7256418e-20))
let res: IResult<&[u8],f32> = be_f32(b"\x1f\x2f\xf0\x3f"); //Ok(([], 3.7256418e-20))
let res: IResult<&str,f32> = float("3.14ab"); //Ok(("ab", 3.14))
let res: IResult<&[u8],u32> = hex_u32(b"123a"); //Ok(([], 4666))
```

### 其它解析器

- `alpha0`: 匹配[a-zA-Z]
- `alphanumeric0`: 匹配[0-9a-zA-Z]，相似功能`alphanumeric1`
- `digit0`: 匹配[0-9]，相似功能`digit1`、`hex_digit0`、`hex_digit1`
- `crlf`: 匹配`\r\n`，类似`line_ending`可匹配`\n` 和 `\r\n`
- `space0`: 匹配空格和`\t`，相似功能`space1`、`multispace0`、`multispace1`
- `tab`: 匹配`\t`
- `rest`: 返回剩余的`input`
- `rest_len`: 返回剩余的`input`的长度
- `value`: 匹配成功则返回给定的值
- `recognize`: 子解析器解析成功则返回子解析器所消耗的值
- `opt`: 包装解析器，使其返回`Option`类型
- `map_res`: 对解析器的输出应用指定的函数
- `verify`: 先使用第二个参数的函数验证，验证成功后返回第一个解析器结果
- `map`: 第一个函数是解析器，第二个是对解析器成功后的结果二次处理函数，功能同`Parser::map`
- `flat_map`: 第一个函数结果作为第二个解析器的入参，返回第二解析器的结果

```rust
let res: IResult<&str, u32> = value(1234, alpha0)("abc001"); //Ok(("001", 1234))
let res: IResult<&str, &str> = recognize(separated_pair(alpha1, char(':'), alpha1))("Jim:Bod007"); //Ok(("007", "Jim:Bod"))
let res: IResult<&str, Option<&str>> = opt(tag("ab"))("acd"); //Ok(("acd", None))
let res: IResult<&str, u32> = map_res(alphanumeric1, |s: &str| u32::from_str_radix(s, 16))("acd0"); //Ok(("", 44240))
let res: IResult<&str, &str> = verify(alphanumeric1, |c: &str| c.starts_with("ab")))("abb"); //Ok(("", "abb"))
let res: IResult<&str, usize> = map(alpha1, |s: &str| s.len())("abcde001"); //Ok(("001", 5))
let res: IResult<&[u8], &[u8]> = flat_map(nom::number::complete::u8, take)(b"\x01\x12\x34\x56"); //Ok(([86], [18, 52]))
```

## 输入数据类型

从上可以看出，`nom`主要处理`&[u8]`和`&str`数据，实际上，要实现特定类型的输入数据，输入数据类型实现相关`trait`即可。具体可以参照[Custom input types](https://github.com/rust-bakery/nom/blob/main/doc/custom_input_types.md)。

## 解析器测试与调试

通过[解析器和组合器](#解析器和组合器)组合可以写出新的解析器，虽然将解析代码直接插入业务逻辑中写起来很爽，但将会导致代码无法维护和测试。好的实践是将解析器放在一个单独的模块中。

测试方法一般的`rust`测试一样，通常可以**将需要测试的数据单独放一个数据文件**，如文本文件或二进制文件，这样可以避免在测试中准备大量重复的数据。

调试的话可以使用`dbg_dmp`：

```rust
fn f(i: &[u8]) -> IResult<&[u8], &[u8]> {
  dbg_dmp(tag("abcd"), "tag")(i)
}

let a = &b"efghijkl"[..];
f(a);
// 输出如下
// tag: Error(Error(Error { import: [101, 102, 103, 104, 105, 106, 107, 108] })) at:
// 00000000        65 66 67 68 69 6a 6b 6c         efghijkl
```

## 错误处理

### 错误输出格式

`nom`的错误管理基于如下原则设计：

- 指示解析器错误的位置
- 指示错误传播链
- 错误消耗低，可被调用解析器丢弃
- 可根据用户需求更改

`nom`解析器返回类型如下：

```rust
pub type IResult<I, O, E=nom::error::Error<I>> = Result<(I, O), nom::Err<E>>;

pub enum Err<E> {
    Incomplete(Needed),
    Error(E),
    Failure(E),
}
```

从上可以看出，解析成功的返回`Ok((I, O))`，其中`I`是输入数据类型，`O`是解析的数据类型。解析错误返回如下三种错误：

- `Incomplete`: 数据不完整
- `Error`: 普通解析错误
- `Failure`: 不可恢复错误

提取非`Incomplete`错误信息代码示范：

```rust
let res: IResult<&[u8],u32> = hex_u32(b"g123a");
let r: Result<(&[u8],u32), nom::error::Error<&[u8]>> = res.finish();
```

其中`nom::error::Error`定义如下：

```rust
#[derive(Debug, PartialEq)]
pub struct Error<I> {
  /// position of the error in the input data
  pub input: I,
  /// nom error code
  pub code: ErrorKind,
}
```

`ErrorKind`指示出现何种错误，其开销比较低，适合频繁调用，但可读性不强，可通过`nom::error::VerboseError`以及`convert_error`将其转换为可读性更强的错误提示。

### 自定义错误

因为`nom`组合器对其错误`trait`处理都是通用的，所以通过实现`ParseError<I>`这个`trait`可以实现自定义错误，该`trait`包含四个方法：

- `from_error_kind`: 从输入数据位置和`ErrorKind`指示哪个解析器错误
- `append`: 通过解析树回溯创建错误链
- `from_char`: 创建解析器期望的字符
- `or`: 允许在各个分支的错误之间进行选择

此外还需要实现`ContextError`这个`trait`以支持`context`方法调用，下面是官方的例子：

```rust
struct DebugError {
    message: String,
}
impl ParseError<&str> for DebugError {
    // on one line, we show the error code and the input that caused it
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        let message = format!("{:?}:\t{:?}\n", kind, input);
        println!("{}", message);
        DebugError { message }
    }

    // if combining multiple errors, we show them one after the other
    fn append(input: &str, kind: ErrorKind, other: Self) -> Self {
        let message = format!("{}{:?}:\t{:?}\n", other.message, kind, input);
        println!("{}", message);
        DebugError { message }
    }

    fn from_char(input: &str, c: char) -> Self {
        let message = format!("'{}':\t{:?}\n", c, input);
        println!("{}", message);
        DebugError { message }
    }

    fn or(self, other: Self) -> Self {
        let message = format!("{}\tOR\n{}\n", self.message, other.message);
        println!("{}", message);
        DebugError { message }
    }
}

impl ContextError<&str> for DebugError {
    fn add_context(input: &str, ctx: &'static str, other: Self) -> Self {
        let message = format!("{}\"{}\":\t{:?}\n", other.message, ctx, input);
        println!("{}", message);
        DebugError { message }
    }
}
```

## 字符测试函数

在前面`take_while(is_alphabetic)`中，使用了字符测试函数来判断，其函数原型为：

```rust
Fn(<Input as InputTakeAtPosition>::Item) -> bool
```

`nom`中提供的函数如下：

- `is_alphabetic`: [A-Za-z]
- `is_alphanumeric`: [A-Za-z0-9]
- `is_digit`: [0-9]
- `is_hex_digit`: [0-9A-Fa-f]
- `is_oct_digit`: [0-7]
- `is_bin_digit`: [0-1]
- `is_space`: [ \t]
- `is_newline`: [\n]

## 一些例子

### C语言注释

```rust
fn c_comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
  value(
    (), delimited(tag("/*"), take_until("*/"), tag("*/"))
  ).parse(input)
}

let res: IResult<&str, ()> = c_comment("/*this is comment*/other"); //Ok(("other"),())
```

### 去掉字符串前后空格

```rust
fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where F: Parser<&'a str, O, E> {
  delimited(multispace0, inner, multispace0)
}

let parser = take_until(" ");
let res: IResult<&str, ()> = ws(parser).parse(" abc "); //Ok(("", "abc"))
```

### 十六进制转换

```rust
fn hex_number<'a>(input: &'a str) -> IResult<&'a str, u32> {
  map_res(
    preceded(
      alt((tag("0x"), tag("0X")))，
      recognize(many1(one_of("0123456789abcdefABCDEF")))
    ), |s: &str| u32::from_str_radix(s, 16)
  ).parse(input)
}

let res: IResult<&str, u32> = hex_number("0xF089"); //Ok(("", 61577))
```

### 实现`FromStr`

函数原型如下：

```rust
pub fn parse<F: FromStr>(&self) -> Result<F, F::Err>
```

也就是说，只要数据结构实现了`std::str::FromStr`这个`trait`，就可以调用对应的`parse`方法，示例如下：

```rust
fn parse_name(input: &str) -> IResult<&str, &str> {
  let (i, _) = tag("Hello, ").parse(input)?;
  let (i, name) = take_while(|c:char| c.is_alphabetic())(i)?;
  let (i, _) = tag("!")(i)?;
  Ok((i, name))
}

struct Name(pub String);

impl FromStr for Name {
  type Err = Error<String>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match parse_name(s) {
      Ok((_, name)) => Ok(Name(name.to_string())),
      Err(_) => todo!(),
    }
  }
}

fn main() {
  let n = "Hello, Jobs!".parse::<Name>();
  println!("result: {:?}", n); //Ok(Name("Jobs"))
}
```

## 参考文献

- [Making a new parser from scratch](https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md)
- [Common recipes to build nom parsers](https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md)
- [List of parsers and combinators](https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md)
- [nom doc](https://docs.rs/nom/latest/nom)
