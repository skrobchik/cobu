#[allow(dead_code)]
fn main() {
    println!("Hello World!");
}

mod mylib {
    pub use self::nested_lib::MyTrait;

    mod nested_lib {
        pub trait MyTrait {
            fn duplicate(self) -> Self;
        }

        impl MyTrait for i32 {
            fn duplicate(self) -> Self {
                self + self
            }
        }
    }
}
