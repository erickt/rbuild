extern crate rbuild;

use rbuild::context::Context;
use rbuild::builders::c::gcc::{StaticBuilder, SharedBuilder};

fn main() {
    let ctx = Context::new();

    let c_static = StaticBuilder::new(ctx.clone())
        .set_debug(true)
        .set_optimize(true);

    let lib = c_static.link_lib("examples/cxx/bar")
        .add_src(
            c_static.compile("examples/cxx/bar.cc"));

    let _exe = c_static.link_exe("examples/cxx/foo_static")
        .add_src(c_static.compile("examples/cxx/foo.cc"))
        .add_lib(lib)
        .run();


    let c_shared = SharedBuilder::new(ctx.clone())
        .set_debug(true)
        .set_optimize(true);

    let lib = c_shared.link_lib("examples/cxx/bar")
        .add_src("examples/cxx/bar.cc");

    let _exe = c_shared.link_exe("examples/cxx/foo_shared")
        .add_src(c_shared.compile("examples/cxx/foo.cc"))
        .add_lib(lib)
        .run();
}
