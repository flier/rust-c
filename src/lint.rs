use data::*;
use types;

use std::{env, path};
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::ffi::OsString;

use syntax::ast::Expr;
use syntax::ast::Expr_::*;
use syntax::ptr::P;

use rustc::lint::*;
use rustc::session::search_paths::SearchPaths;

use gcc;

fn super_hack_get_out_dir() -> OsString {
    let mut out_dir = None;
    let mut found_out_dir = false;
    for a in env::args() {
        if a == "--out-dir" {
            found_out_dir = true;
            continue;
        }
        if found_out_dir {
            out_dir = Some(OsString::from(format!("{}", a)));
            break;
        }
    }

    out_dir.unwrap_or_else(|| {
        env::current_dir().unwrap().into_os_string()
    })
}

/// This lint pass right now doesn't actually do any linting, instead it has the role
/// of building the library containing the code we just extracted with the macro,
/// before rustc tries to link against it.
///
/// At some point, I'd like to use this phase to gain type information about the captured
/// variables, and return values, and use that to ensure the correctness of the code, and
/// provide nicer types etc. to the c++ code which the user writes.
///
/// For example, #[repr(C)] structs could have an equivalent type definition generated in
/// the C++ side, which could allow for some nicer interaction with the captured values.
pub struct CppLintPass;
impl LintPass for CppLintPass {
    fn get_lints(&self) -> LintArray {
        lint_array!()
    }

    fn check_expr(&mut self, cx: &Context, exp: &Expr) {
        if let ExprCall(ref callee, ref args) = exp.node {
            if let ExprPath(None, ref path) = callee.node {
                if path.segments.len() == 1 {
                    let name = path.segments[0].identifier.name.as_str();

                    record_type_data(cx, name, exp, args);
                }
            }
        }
    }
}

fn record_type_data(cx: &Context, name: &str, call: &Expr, args: &[P<Expr>]) {
    let mut headers = CPP_HEADERS.lock().unwrap();
    let mut decls = CPP_FNDECLS.lock().unwrap();
    let mut types = CPP_TYPEDATA.lock().unwrap();

    if let Some(cppfn) = decls.get_mut(name) {
        match types::cpp_type_of(&mut types, cx.tcx, call, false) {
            Ok(ty) => cppfn.ret_ty = Some(ty),

            // XXX FIXME => this shouldn't panic
            Err(reason) => panic!("Invalid return type: {}", reason),
        }

        for (i, arg) in args.iter().enumerate() {
            // Strip the two casts off
            if let ExprCast(ref e, _) = arg.node {
                if let ExprCast(ref e, _) = e.node {
                    if let ExprAddrOf(_, ref e) = e.node {
                        match types::cpp_type_of(&mut types, cx.tcx, e, true) {
                            Ok(ty) => cppfn.arg_idents[i].ty = Some(ty),

                            // XXX FIXME => this shouldn't panic
                            Err(reason) => panic!("Invalid arg type: {}", reason),
                        }

                        continue
                    }
                }
            }

            panic!("Expected a double-casted reference as an argument.")
        }
    } else { return }

    // We've processed all of them!
    // Finalize!
    if decls.values().all(|x| x.ret_ty.is_some()) {
        finalize(cx, &mut headers, &mut types, &mut decls);
    }
}

fn finalize(cx: &Context,
            headers: &mut String,
            types: &mut types::TypeData,
            decls: &mut HashMap<String, CppFn>) {
    let fndecls = decls.values().fold(String::new(), |acc, new| {
        format!("{}\n{}\n", acc, new.to_string())
    });

    let cppcode = format!(r#"
/******************************
 * Code Generated by Rust-C++ *
 ******************************/

/* cstdint includes sane type definitions for integer types */
#include <cstdint>

/* the rs:: namespace contains rust-defined types */
namespace rs {{
    /* A slice from rust code */
    /* Can be used to interact with, pass around, and return Rust slices */
    template<class T>
    struct Slice {{
        const T*  data;
        uintptr_t len;
    }};

    /* A string slice is simply a slice of utf-8 encoded characters */
    typedef Slice<uint8_t> StrSlice;

    /* A trait object is composed of a data pointer and a vtable */
    struct TraitObject {{
        void* data;
        void* vtable;
    }};
}}

/* User-generated Headers */
{}

/* Generated types */
{}

/* User-generated function declarations */
extern "C" {{{}}}
"#, *headers, types.to_cpp(), fndecls);

    // Get the output directory, which is _way_ harder than I was expecting,
    // (also super hacky).
    let out_dir = super_hack_get_out_dir();

    // Create the C++ file which we will compile
    {
        let path = path::Path::new(&out_dir).join("rust_cpp_tmp.cpp");
        let mut f = File::create(path).unwrap();
        f.write_all(cppcode.as_bytes()).unwrap();
    }


    // Unfortunately, once the compiler is running, which it is (as we are running
    // inside of it), there doesn't appear to be a way to add SearchPaths for library
    // lookup.
    // To get around this, this unsafe block takes the shared reference to the parsed
    // options SearchPaths object, casts it to a mutable reference, and adds a path
    // to the SearchPaths object through that mutable reference.
    // The rustc backend hasn't read from this object into it's own internal storage
    // for linking yet, so this change will be read and used in the linking phase.
    unsafe {
        let sp = &cx.sess().opts.search_paths;
        let sp_mut: &mut SearchPaths = &mut *(sp as *const _ as *mut _);
        sp_mut.add_path(&out_dir.to_str().unwrap());
    }

    // I didn't want to write my own compiler driver-driver, so I'm using gcc.
    // Unfortuantely, it expects to be run within a cargo build script, so I'm going
    // to set a bunch of environment variables to trick it into not crashing
    env::set_var("TARGET", &cx.sess().target.target.llvm_target);
    env::set_var("HOST", &cx.sess().host.llvm_target);
    env::set_var("OPT_LEVEL", format!("{}", cx.sess().opts.cg.opt_level.unwrap_or(0)));
    env::set_var("CARGO_MANIFEST_DIR", &out_dir);
    env::set_var("OUT_DIR", &out_dir);
    env::set_var("PROFILE", "");
    env::set_var("CXXFLAGS", format!("{} {}", env::var("CXXFLAGS").unwrap_or(String::new()),
                                     "-std=c++11"));

    println!("########### Running GCC ###########");
    gcc::Config::new()
        .cpp(true)
        .file("rust_cpp_tmp.cpp")
        .compile("librust_cpp_tmp.a");
    println!("########### Done Rust-C++ ############");
}
