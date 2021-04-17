//! This library provides two key pieces:
//!
//! -   An implementation of `GhostCell` and `GhostToken`, as per http://plv.mpi-sws.org/rustbelt/ghostcell/.
//! -   A new `GhostSea` type, attempting to make the use of the above more ergonomic.
//!
//! #   Safety
//!
//! http://plv.mpi-sws.org/rustbelt/ghostcell/ left some blanks in the implementation of `GhostCell` and `GhostToken`
//! that I have filled in myself. I hopefully didn't make a mistake, hopefully.
//!
//! `GhostSea` is on even shakier ground. It _looks_ about right to my eye, but that's not near as good as a formal
//! proof.
//!
//! As a result, the reader is invited to carefully scrutinize the code of this library and consciously decide whether
//! it looks good enough -- and safe enough -- for them.
//!
//! Use at your own risk!

//  Generic features.
#![cfg_attr(not(test), no_std)]

//  Lints.
#![deny(missing_docs)]

mod ghost_sea;

pub use self::ghost_sea::*;
