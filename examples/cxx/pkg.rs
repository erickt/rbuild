extern crate rbuild;

use rbuild::context::Context;
use rbuild::c::gcc::SharedBuilder;

fn main() {
    let ctx = Context::new();
    let compiler = SharedBuilder::new(ctx.clone(), "/usr/bin/gcc");

    let lib = compiler.link_lib("examples/cxx/bar")
        .add_src("examples/cxx/bar.cc");

    let _exe = compiler.link_exe("examples/cxx/foo")
        .add_src(compiler.compile("examples/cxx/foo.cc"))
        .add_lib(lib)
        .run();
}
