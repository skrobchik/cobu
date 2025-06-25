#[allow(dead_code)]
fn main() {
    println!("{}", add(2, 2));
}

use mylib::add;

mod mylib {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}
