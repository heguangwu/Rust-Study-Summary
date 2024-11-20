# `const` 的使用注意事项

在Rust中， `const` 在 `Rust` 用于定义常量，看起来和 `let` 很像，但实际使用时存在一些易出错的地方，这篇文章主要讲述 `const` 的使用陷阱。

## 基本使用

`let` 使用：`let PAT = EXPR`

`const` 使用： `const IDENT: TYPE = EXPR`

也就是说:

>***`const` 使用时必须指定类型***。

## 陷阱一: `const` 声明会被提升

看下面代码：

```rust
fn hi() -> i32 {
    return Y;
    const Y: i32 = X * X;
    const X: i32 = 5;
}

fn main() {
    let r = hi();
    println!("r={r}");
}
```

这段代码可以编译通过且正常运行。

从这里我们可以猜测：
>***`const` 是在编译过程中完成符号替换的，所以在返回Y之前Y并没有声明和定义，Y的计算时X同样没有声明和定义***

## 陷阱二：`match` 模式匹配 `const`

在 `match` 匹配时，未定义的 `const` 常量不会报错且总是匹配成功

```rust
const GOOD:i32 = 1;
const BAD:i32 = 0;

fn main() {
    let input = 2;
    match input {
        BAD => println!("is BAD"),
        GOD => println!("is GOOD"),
        _ => println!("UNKNOWN")
    }
}
```

上述程序中的匹配项 `GOOD` 错误的写成了 `GOD` ，但出人意料的是运行结果输出 `is GOOD`，从这里我们可以猜测：
>***当`match`对应的项 `GOD` 没有定义时， `GOD` 被当做一个变量被赋值，从而匹配，这种情况尤其在匹配Enum类型时要注意拼写错误***

```rust
let (a, b) = (5, 2);
let (5, x) = (a, b) else {panic!("not match")};
```

## 总结

这里可以直接看一个参考资料给的例子：

```rust
macro_rules! f {
    ($cond: expr) => {
        if let Some(x) = $cond {
        println!("i am some == {x}!");
        } else {
        println!("i am none");
        }
    }
}

fn main() {
    f!(Some(100));
    {
        f!(Some(100));
        return;
        const x: i32 = 5;
    }
}
```

输出结果为：

```shell
i am some == 100!
i am none
```

如果你想明白了，那么这篇文章就算对你有所帮助了。

## 参考文献

[Rust's Most Subtle Syntax](https://zkrising.com/writing/rusts-most-subtle-syntax/)
