extern crate c;

fn main() {
    c::build("src/lib.rs", "c_test", |cfg| {
    });
}
