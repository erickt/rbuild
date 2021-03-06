use std::io;
use std::io::fs;
use sync::Future;

use context::{Context, Call};
use into_path::IntoPath;
use into_future::IntoFuture;
use path_util;

#[deriving(Clone)]
pub struct Ar {
    ctx: Context,
    exe: Path,
    dst_prefix: Option<&'static str>,
    dst_suffix: Option<&'static str>,
    dst: Option<Path>,
    srcs: Vec<Path>,
    flags: Vec<~str>,
}

static EXES: &'static [&'static str] = &["ar"];

impl Ar {
    pub fn new(ctx: Context) -> Ar {
        let exe = path_util::find_program(ctx.clone(), EXES);
        Ar::new_with(ctx, exe)
    }

    pub fn new_with<T: IntoFuture<Path>>(ctx: Context, exe: T) -> Ar {
        let mut flags = Vec::new();
        flags.push(~"-rc");
        Ar {
            ctx: ctx,
            exe: exe.into_future().unwrap(),
            dst_prefix: None,
            dst_suffix: None,
            dst: None,
            srcs: Vec::new(),
            flags: flags,
        }
    }

    pub fn set_dst_prefix(mut self, dst_prefix: &'static str) -> Ar {
        self.dst_prefix = Some(dst_prefix);
        self
    }

    pub fn set_dst_suffix(mut self, dst_suffix: &'static str) -> Ar {
        self.dst_suffix = Some(dst_suffix);
        self
    }

    pub fn set_dst<T: IntoPath>(mut self, dst: T) -> Ar {
        let mut dst = dst.into_path();

        // Make sure we write the output in the build/ directory.
        if !dst.is_ancestor_of(&self.ctx.root) {
            dst = self.ctx.root.join(dst);
        }

        self.dst = Some(dst);
        self
    }

    pub fn add_src<T: IntoFuture<Path>>(mut self, src: T) -> Ar {
        self.srcs.push(src.into_future().unwrap());
        self
    }

    pub fn add_flag<T: Str>(mut self, flag: T) -> Ar {
        self.flags.push(flag.into_owned());
        self
    }

    pub fn run(self) -> Path {
        self.into_future().unwrap()
    }
}

impl IntoFuture<Path> for Ar {
    fn into_future(self) -> Future<Path> {
        let Ar {
            ctx,
            exe,
            dst_prefix,
            dst_suffix,
            dst,
            srcs,
            flags
        } = self;

        assert!(dst.is_some());
        let mut dst = dst.unwrap();
        dst = path_util::add_prefix_suffix(dst, dst_prefix, dst_suffix);

        let mut call = Call::new(exe.clone()).unwrap();

        for flag in flags.move_iter() {
            call.push_str(flag);
        }

        call.push_output_path(dst.clone());

        for src in srcs.iter() {
            call.push_input_path(src.clone()).unwrap();
        }

        let mut prep = ctx.prep("Call");
        prep.declare_call(&call);

        prep.exec(proc(exec) {
            let (prog, args) = call.cmd();

            // Make sure the parent directories exist.
            fs::mkdir_recursive(&dst.dir_path(), io::UserDir).unwrap();

            let status = exec.process_builder(prog, args.as_slice())
                .description(exe.filename_display())
                .msg(dst.display())
                .msg("<-")
                .msgs(srcs.iter().map(|src| src.display()))
                .run()
                .unwrap();

            if !status.success() {
                fail!("command failed");
            }

            dst
        })
    }
}
