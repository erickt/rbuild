extern crate rbuild;

use rbuild::context::Context;
use rbuild::c::gcc::{Gcc, SharedBuilder};

fn main() {
    let ctx = Context::new();

    let builder = SharedBuilder::new_with(
        Gcc::new(ctx.clone(), "/usr/bin/gcc")
            .set_debug(true)
            .set_optimize(true));

    let lib = builder.link_lib("examples/cxx/bar")
        .add_src("examples/cxx/bar.cc");

    let _exe = builder.link_exe("examples/cxx/foo")
        .add_src(builder.compile("examples/cxx/foo.cc"))
        .add_lib(lib)
        .run();
}
