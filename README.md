# rust-c

Forked from [rust-cpp](https://github.com/mystor/rust-cpp) version 0.1.0 for situations where a full C++ compiler isn't available or warranted. Pretty hacky implementation, intended to get a few bits and pieces bootstrapped until Rust matures enough that some C code can be replaced. I couldn't have written this myself from scratch, so hats off to @mystor. One day this crate may disappear and be subsumed into [rust-cpp](https://github.com/mystor/rust-cpp).

Comments that follow are from [rust-cpp](https://github.com/mystor/rust-cpp) with minor changes of C++ and `cpp` to C and `c`, etc.

rust-c is a build tool & macro which enables you to write C code inline in
your rust code.

> NOTE: This crate works on stable rust, but it is not stable itself. You can
> use this version all you want, but don't be surprised when a 0.2 release is
> made which completely breaks backwords compatibility. I view this crate as
> more of an experiment than a product.

> As the tools come into stable rust to make this more practical to use, I
> expect that it will stabilize. Namely, I do not expect that this module will
> have a stable-ish interface until we get a stable procedural macro system.

## Setup

Add `c` as a dependency to your project. It will need to be added both as a
build dependency, and as a normal dependency, with different flags. You'll also
need a `build.rs` set up for your project.

```toml
[package]
# ...
build = "build.rs"

[build-dependencies]
# ...
c = { version = "0.1.0", features = ["build"] }

[dependencies]
# ...
c = { version = "0.1.0", features = ["macro"] }
```

You'll also then need to call the `c` build plugin from your `build.rs`. It
should look something like this:

```rust
extern crate c;

fn main()
{
    c::build("src/lib.rs", "crate_name", |cfg|
    {
        // cfg is a gcc::Config object. You can use it to add additional
        // configuration options to the invocation of the C compiler.
    });
}
```

## Usage

In your crate, include the cpp crate macros:

```rust
#[macro_use]
extern crate c;
```

Then, use the `c!` macro to define code and other logic which you want shared
between rust and C. The `c!` macro supports the following forms:

```rust
c!
{
    // Include a C header into the C shim. Only the `#include` directive 
    // is supported in this context.
    #include <stdlib.h>
    #include "foo.h"
    
    // Write some logic directly into the shim. Either a curly-braced block or
    // string literal are supported
    raw
    {
        #define X 10
        struct Foo
        {
            uint32_t x;
        };
    }
    
    raw r#"
        #define Y 20
    "#
    
    // Define a function which can be called from rust, but is implemented in
    // C. Its name is used as the C function name, and cannot collide with
    // other C functions. The body may be defined as a curly-braced block or 
    // string literal.
    // These functions are unsafe, and can only be called from unsafe blocks.
    fn my_function(x: i32 as "int32_t", y: u64 as "uint32_t") -> f32 as "float"
    {
        return (float)(x + y);
    }
    fn my_raw_function(x: i32 as "int32_t") -> u32 as "uint32_t" r#"
        return x;
    "#
    
    // Define a struct which is shared between C and rust. In C-land its
    // name will be in the global namespace (there's only one)! In rust it will be located 
    // wherever the c! block is located
    struct MyStruct
    {
        x: i32 as "int32_t",
        y: *const i8 as "const char*",
    }
    
    // Define an enum which is shared between C and rust. In C-land it 
    // will be defined in the global namespace as an `enum` (there's only one)!. In rust,
    // it will be located wherever the c! block is located.
    enum MyEnum
    {
        A, // Known in C as `A`
        B,
        C,
        D,
    }
}
```

`c` also provides a header which may be useful for interop code. This header
includes `<stdint.h>`. This header, `rust_types.h`, can be included with:-

```rust
c!
{
    #include "rust_types.h"
}
```

The full body of `rust_types.h` is included below.

```c
#ifndef _RUST_TYPES_H_
#define _RUST_TYPES_H_

#include <stdint.h>

typedef int8_t i8;
typedef int16_t i16;
typedef int32_t i32;
typedef int64_t i64;
typedef intptr_t isize;

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef uintptr_t usize;

typedef float f32;
typedef double f64;

typedef u8 bool_;

typedef uint32_t char_;

#endif
```

## Warning about Macros

rust-cpp cannot identify and parse the information found in cpp! blocks which
are generated with macros. These blocks will correctly generate rust code, but
will not generate the corresponding C++ code, most likely causing your build to
fail with a linker error. Do not create `cpp! {}` blocks with macros to avoid
this.
