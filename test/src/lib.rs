#![cfg_attr(not(test), allow(dead_code))]
#![allow(improper_ctypes)]

#[macro_use]
extern crate c;

#[cfg(test)]
use std::ffi::CString;

c! {
    // Bring in rust-types!
    #include "rust_types.h"
}

c! {
    fn basic_math_impl(a: i32 as "rs::i32", b: i32 as "rs::i32") -> i32 as "rs::i32" {
        int32_t c = a * 10;
        int32_t d = b * 20;

        return c + d;
    }
}

#[test]
fn basic_math() {
    let a: i32 = 10;
    let b: i32 = 20;

    let c_result = unsafe {
        basic_math_impl(a, b)
    };

    assert_eq!(c_result, 500);
    assert_eq!(a, 10);
    assert_eq!(b, 20);
}

c! {
    fn strings_impl(local_cstring: *mut u8 as "char *") {
        local_cstring[3] = 'a';
    }
}

#[test]
fn strings() {
    let cs = CString::new(&b"Hello, World!"[..]).unwrap();

    unsafe {
        strings_impl(cs.as_ptr() as *mut _);
    }

    assert_eq!(cs.as_bytes(), b"Helao, World!");
}

#[cfg(test)]
mod inner;

#[test]
fn inner_module() {
    let x = inner::inner();
    assert_eq!(x, 10);
}

c! {
    #include <math.h>
    fn c_std_lib_impl(num1: f32 as "float",
                      num2: f32 as "float")
                      -> f32 as "float" {
        return sqrt(num1) + cbrt(num2);
    }
}

#[test]
fn c_std_lib() {
    let num1: f32 = 10.4;
    let num2: f32 = 12.5;

    unsafe {
        let res = c_std_lib_impl(num1, num2);

        let res_rs = num1.sqrt() + num2.cbrt();

        assert!((res - res_rs).abs() < 0.001);
    }
}

c! {
    #[derive(PartialEq, Eq, Debug)]
    enum Foo {
        Apple,
        Peach,
        Cucumber,
    }

    fn basic_enum_impl_1(foo: Foo as "Foo", bar: Foo as "Foo", quxx: Foo as "Foo")
                         -> bool as "bool"
    {
        return foo == Apple && bar == Peach && quxx == Cucumber;
    }

    fn basic_enum_impl_2() -> Foo as "Foo" {
        return Cucumber;
    }
}

#[test]
fn basic_enum() {
    let foo = Foo::Apple;
    let bar = Foo::Peach;
    let quxx = Foo::Cucumber;

    unsafe {
        assert!(basic_enum_impl_1(foo, bar, quxx));

        let returned_enum = basic_enum_impl_2();
        assert_eq!(returned_enum, Foo::Cucumber);
    }
}

c! {
    raw {
        #define SOME_CONSTANT 10
    }

    fn return_some_constant() -> u32 as "uint32_t" {
        return SOME_CONSTANT;
    }
}

#[test]
fn header() {
    unsafe {
        let c = return_some_constant();
        assert_eq!(c, 10);
    }
}

c! {
    #[derive(Copy, Clone)]
    struct S {
        a: i32 as "int32_t",
    }
}

#[test]
fn derive_copy() {
    let x = S { a: 10 };
    let mut y = x;
    assert_eq!(x.a, 10);
    assert_eq!(y.a, 10);
    y.a = 20;
    assert_eq!(x.a, 10);
    assert_eq!(y.a, 20);
}

c! {
    raw "#define SOME_VALUE 10"

    fn string_body_impl() -> i32 as "int32_t" r#"
        return SOME_VALUE;
    "#
}

#[test]
fn string_body() {
    unsafe {
        assert_eq!(string_body_impl(), 10);
    }
}
