use std::io;
use std::io::process::Process;
use std::io::fs;
use sync::Future;

use context::{Context, Call};
use into_path::IntoPath;
use into_future::IntoFuture;

#[deriving(Clone)]
pub struct SharedBuilder {
    ctx: Context,
    exe: Path,
    flags: ~[~str],
}

impl SharedBuilder {
    pub fn new<T: IntoPath>(ctx: Context, exe: T) -> SharedBuilder {
        SharedBuilder {
            ctx: ctx,
            exe: exe.into_path(),
            flags: ~[],
        }
    }

    pub fn compile<T: IntoFuture<Path>>(&self, src: T) -> Gcc {
        let src = src.into_future().unwrap();
        let dst = src.with_extension("o");

        Gcc::new(self.ctx.clone(), self.exe.clone(), dst)
            .add_src(src)
            .add_flag(~"-c")
            .add_flag(~"-fPIC")
    }

    pub fn link_lib<'a, T: IntoPath>(&self, dst: T) -> Gcc {
        let mut dst = dst.into_path();

        // change the filename to be "lib${filename}.dylib".
        let filename = format!("lib{}.dylib", dst.filename_str().unwrap());
        dst.set_filename(filename);

        Gcc::new(self.ctx.clone(), self.exe.clone(), dst)
            .add_flag(~"-fPIC")
            .add_flag(~"-dynamiclib")
    }

    pub fn link_exe<'a, T: IntoPath>(&self, dst: T) -> Gcc {
        Gcc::new(self.ctx.clone(), self.exe.clone(), dst.into_path())
    }
}

pub struct Gcc {
    priv ctx: Context,
    priv exe: Path,
    priv dst: Path,
    priv srcs: ~[Path],
    priv includes: ~[Path],
    priv libs: ~[Path],
    priv flags: ~[~str],
}

impl Gcc {
    pub fn new(ctx: Context, exe: Path, mut dst: Path) -> Gcc {
        // Make sure we write the output in the build/ directory.
        if !dst.is_ancestor_of(&ctx.root) {
            dst = ctx.root.join(dst);
        }

        Gcc {
            ctx: ctx,
            exe: exe,
            dst: dst,
            srcs: ~[],
            includes: ~[],
            libs: ~[],
            flags: ~[],
        }
    }

    pub fn add_src<T: IntoFuture<Path>>(mut self, src: T) -> Gcc {
        self.srcs.push(src.into_future().unwrap());
        self
    }

    pub fn add_srcs<T: IntoFuture<Path>>(mut self, srcs: ~[T]) -> Gcc {
        let mut iter = srcs.move_iter().map(|src| src.into_future().unwrap());
        self.srcs.extend(&mut iter);
        self
    }

    pub fn add_include<T: IntoFuture<Path>>(mut self, include: T) -> Gcc {
        self.includes.push(include.into_future().unwrap());
        self
    }

    pub fn add_lib<T: IntoFuture<Path>>(mut self, lib: T) -> Gcc {
        let lib = lib.into_future().unwrap();
        self.libs.push(lib);
        self
    }

    pub fn add_flag<S: Str>(mut self, flag: S) -> Gcc {
        self.flags.push(flag.as_slice().to_owned());
        self
    }

    pub fn run(self) -> Path {
        self.into_future().unwrap()
    }
}

impl IntoFuture<Path> for Gcc {
    fn into_future(self) -> Future<Path> {
        let Gcc { ctx, exe, dst, srcs, includes, libs, flags } = self;

        assert!(!srcs.is_empty());

        let mut prep = ctx.prep("Call");

        let mut call = Call::new(exe.clone()).unwrap();

        call.push_str(~"-o");
        call.push_output_path(dst.clone());

        for include in includes.move_iter() {
            call.push_str(~"-I");
            call.push_input_path(include).unwrap();
        }

        let mut new_libs: ~[~str] = ~[];
        let mut new_libpaths: ~[Path] = ~[];

        for lib in libs.move_iter() {
            prep.declare_input_path(lib.clone()).unwrap();

            new_libpaths.push(lib.dir_path());

            let name = lib.filename_str().unwrap();

            let prefix = "lib";
            let suffix = ".dylib";

            assert!(name.starts_with(prefix) && name.ends_with(suffix));

            new_libs.push(name.slice(prefix.len(), suffix.len()).to_owned());
        }

        for libpath in new_libpaths.move_iter() {
            call.push_str(~"-L");
            call.push_str(libpath.as_str().unwrap().to_owned());
        }

        for lib in new_libs.move_iter() {
            call.push_str(~"-l");
            call.push_str(lib);
        }

        for flag in flags.move_iter() {
            call.push_str(flag);
        }

        for src in srcs.iter() {
            call.push_input_path(src.clone()).ok().expect("src");
        }

        prep.declare_call(&call);

        prep.exec(proc(_exec) {
            let (prog, args) = call.cmd();

            print!("{}:", exe.display());

            for src in srcs.iter() {
                print!(" {}", src.display());
            }

            // Make sure the parent directories exist.
            fs::mkdir_recursive(&dst.dir_path(), io::UserDir).unwrap();

            println!(" -> {}", dst.display());
            println!("{} {}", prog, args);

            let mut process = Process::new(prog, args).unwrap();
            let status = process.wait();

            if !status.success() {
                fail!("command failed");
            }

            dst
        })
    }
}
