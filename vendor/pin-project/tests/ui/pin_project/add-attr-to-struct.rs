use auxiliary_macros::add_attr;
use pin_project::pin_project;
use std::marker::PhantomPinned;

#[pin_project]
#[add_attr(struct)] //~ ERROR duplicate #[pin] attribute
struct Foo {
    #[pin]
    field: PhantomPinned,
}

#[add_attr(struct)] //~ ERROR #[pin] attribute may only be used on fields of structs or variants
#[pin_project]
struct Bar {
    #[pin]
    field: PhantomPinned,
}

fn main() {}
