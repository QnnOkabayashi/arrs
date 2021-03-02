pub mod array;
pub mod tests;

/*
https://stackoverflow.com/a/57259339/12401179
enable no-aliasing (Fortran style)
rust nightly: -Zmutable-noalias=yes
very risky tho, only do it once everything works
*/
