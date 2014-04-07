#![crate_id = "rbuild#0.1.0-pre"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![feature(phase)]

extern crate collections;
extern crate serialize;
extern crate sync;

#[phase(syntax, link)]
extern crate log;

pub mod builders;
pub mod context;
pub mod into_future;
pub mod into_path;
pub mod workcache;
pub mod path_util;
