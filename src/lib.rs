#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

#[macro_use]
pub mod array;
pub mod tests;

/*
https://stackoverflow.com/a/57259339/12401179
enable no-aliasing (Fortran style)
rust nightly: -Zmutable-noalias=yes
very risky tho, only do it once everything works

https://doc.rust-lang.org/edition-guide/rust-2018/error-handling-and-panics/aborting-on-panic.html
smaller binaries by not unwinding panics
*/
