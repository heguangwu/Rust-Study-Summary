# `Future` 的大小

## 如何测量 `rust` `Future` 的大小

通过标准库的`std::mem::size_of`可以获得`Future` 的大小

```rust
fn get_future_size<F: Future>(_fut: F) -> usize {
    std::mem::size_of::<F>()
}
```

## 为什么要关注 `Future` 的大小

`Future` 是一个状态机，默认是放在栈上，而每一个线程的栈空间是比较有限的，默认是2MB。如果一个很大的 `Future` 可能会导致栈溢出。如下程序：

```rust
use std::future::Future;

async fn nothing() {}
async fn huge() {
    let mut a = [0_u8; 20_000];
    nothing().await;
    for (idx, item) in a.iter_mut().enumerate() {
        *item = (idx % 256) as u8;
    }
}

fn main() {
    let f = huge();
    let size = get_future_size(f);
    println!("huge size: {size}");
}
```

>上面代码要注意的是 `nothing()` 必须要 `await`，否则 `nothing()` 就不会执行，这样导致 `huge` 就没有上下文信息，只是一个字节的 `Future` 大小。

输出结果为：

```shell
huge size: 20002
```

其中：`nothing().await` 占用2个字节，数组大小为20000字节。

## 超大的 `Future` 怎么办

这样看来，肯定有超过栈大小的`Future` ，这时候就要使用 `Box` 将 `Future` 移动到堆上，实际上 `tokio` 就是使用自动装箱技术，将超过16KB（Debug模式下位2KB）的 `Future` 移动到堆上，并可以通过 `tokio console` 工具查看。

## 参考文献

[how big is your future?](https://hegdenu.net/posts/how-big-is-your-future/)
