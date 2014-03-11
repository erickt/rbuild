extern crate rbuild;

use rbuild::context::Context;
use rbuild::c::gcc::Builder;

fn main() {
    let ctx = Context::new(Path::new("build/db.json"));
    let compiler = Builder::new(ctx.clone(), "/usr/bin/gcc");
    let objs = ~[
        compiler.compile("examples/cxx/foo.cc").run(),
        compiler.compile("examples/cxx/bar.cc").run(),
    ];
    let _exe = compiler.link_exe("examples/cxx/foo", objs).run().unwrap();
}
