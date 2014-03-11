extern crate rbuild;

use rbuild::context::Context;
use rbuild::c::gcc::SharedBuilder;

fn main() {
    let ctx = Context::new();
    let compiler = SharedBuilder::new(ctx.clone(), "/usr/bin/gcc");
    let objs = ~[
        compiler.compile("examples/cxx/foo.cc").run(),
        compiler.compile("examples/cxx/bar.cc").run(),
    ];
    let _exe = compiler.link_exe("examples/cxx/foo", objs).run().unwrap();
}
