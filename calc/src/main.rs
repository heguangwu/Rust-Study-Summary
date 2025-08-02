use crate::calc::calculate;

mod calc;

fn main() {
    let v = calculate("1+2*3-4^2");
    println!("{:?}", v);
}
