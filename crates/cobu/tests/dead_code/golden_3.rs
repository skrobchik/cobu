mod foo {
    pub(crate) fn foo_used() {}
}

#[allow(dead_code)]
fn main() {
    foo::foo_used();
}
