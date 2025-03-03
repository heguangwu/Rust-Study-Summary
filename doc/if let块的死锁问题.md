# `if let`块的死锁问题

## 基本概念

`rust`在多线程和内存错误捕捉提供很大的帮助，但是编译器并不能捕捉死锁的问题，如下的代码将出现死锁，但编译器无法捕捉：

```rust
use std::sync::RwLock;

fn main() {
    let optional = RwLock::new(Some(123));
    let lock = optional.read().unwrap();
    // some operator
    let lock = optional.write().unwrap();
    println!("Finished!");
}
```

当然上面的代码比较简单，我们只需要在使用之后隐式或显式释放锁即可，如下两种方式。

方法一：显式释放

```rust
use std::sync::RwLock;

fn main() {
    let optional = RwLock::new(Some(123));
    let lock = optional.read().unwrap();
    drop(lock);
    let lock = optional.write().unwrap();
    println!("Finished!");
}
```

方法二：隐式释放

```rust
use std::sync::RwLock;

fn main() {
    let optional = RwLock::new(Some(123));
    {
        let lock = optional.read().unwrap();
    }

    {
        let lock = optional.write().unwrap();
    }
    println!("Finished!");
}
```

## `if let` 块中的死锁

看下面的代码：

```rust
use std::sync::RwLock;

fn main() {
    let map: RwLock<Option<u32>> = RwLock::new(None);
    if let Some(num) = *map.read().unwrap() {
        println!("There's a number in there: {num:?}");
    } else {
        let mut lock2 = map.write().unwrap();
        *lock2 = Some(5);
        println!("There will now be a number {lock2:?}");
    }
    println!("Finished!");
}
```

乍一看好像没啥问题，运行的时候就会发现出现死锁，这是因为上述的代码等同于：

```rust
use std::sync::RwLock;

fn main() {
    let map: RwLock<Option<u32>> = RwLock::new(None);
    {
        let num = map.read().unwrap();
        if num.is_some() {
            println!("There's a number in there: {num:?}");
        } else {
            let mut lock2 = map.write().unwrap();
            *lock2 = Some(5);
            println!("There will now be a number {lock2:?}");
        }
    }
    println!("Finished!");
}
```

>***`if let` 会保持其持有的变量，即使在 `else` 分支也是可以访问的，所以除非后续不会再获取锁，否则尽量不要在 `if let` 中获取锁***

>***该问题在`Rust 2024` 版本已经不存在，详情见[参考文献](https://doc.rust-lang.org/nightly/edition-guide/rust-2024/temporary-if-let-scope.html)***

## 参考资料

[Rust's Sneaky Deadlock With `if let` Blocks](https://brooksblog.bearblog.dev/rusts-sneaky-deadlock-with-if-let-blocks/)

[The Rust Edition Guide](https://doc.rust-lang.org/nightly/edition-guide/rust-2024/temporary-if-let-scope.html)
