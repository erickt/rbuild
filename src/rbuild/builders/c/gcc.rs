use std::io;
use std::io::process::Process;
use std::io::fs;
use sync::Future;

use context::{Context, Call};
use into_path::IntoPath;
use into_future::IntoFuture;

#[deriving(Clone)]
pub struct SharedBuilder {
    priv gcc: Gcc,
}

impl SharedBuilder {
    pub fn new<T: IntoPath>(ctx: Context, exe: T) -> SharedBuilder {
        SharedBuilder::new_with(Gcc::new(ctx, exe))
    }

    pub fn new_with(gcc: Gcc) -> SharedBuilder {
        SharedBuilder {
            gcc: gcc,
        }
    }

    pub fn compile<T: IntoFuture<Path>>(&self, src: T) -> Gcc {
        let src = src.into_future().unwrap();
        let dst = src.with_extension("o");

        self.gcc.clone()
            .set_dst(dst)
            .add_src(src)
            .add_flag(~"-c")
            .add_flag(~"-fPIC")
    }

    pub fn link_lib<T: IntoPath>(&self, dst: T) -> Gcc {
        let mut dst = dst.into_path();

        // change the filename to be "lib${filename}.dylib".
        let filename = format!("lib{}.dylib", dst.filename_str().unwrap());
        dst.set_filename(filename);

        self.gcc.clone()
            .set_dst(dst)
            .add_flag(~"-fPIC")
            .add_flag(~"-dynamiclib")
    }

    pub fn link_exe<T: IntoPath>(&self, dst: T) -> Gcc {
        self.gcc.clone()
            .set_dst(dst)
    }
}

#[deriving(Clone)]
pub struct Gcc {
    priv ctx: Context,
    priv exe: Path,
    priv dst: Option<Path>,
    priv srcs: ~[Path],
    priv includes: ~[Path],
    priv libs: ~[Path],
    priv external_libs: ~[~str],
    priv libpaths: ~[Path],
    priv macros: ~[~str],
    priv warnings: ~[~str],
    priv debug: bool,
    priv profile: bool,
    priv optimize: bool,
    priv flags: ~[~str],
}

impl Gcc {
    pub fn new<T: IntoPath>(ctx: Context, exe: T) -> Gcc {
        Gcc {
            ctx: ctx,
            exe: exe.into_path(),
            dst: None,
            srcs: ~[],
            includes: ~[],
            libs: ~[],
            external_libs: ~[],
            libpaths: ~[],
            macros: ~[],
            warnings: ~[],
            debug: false,
            profile: false,
            optimize: false,
            flags: ~[],
        }
    }

    pub fn set_dst<T: IntoPath>(mut self, dst: T) -> Gcc {
        let mut dst = dst.into_path();

        // Make sure we write the output in the build/ directory.
        if !dst.is_ancestor_of(&self.ctx.root) {
            dst = self.ctx.root.join(dst);
        }

        self.dst = Some(dst);
        self
    }

    pub fn add_src<T: IntoFuture<Path>>(mut self, src: T) -> Gcc {
        self.srcs.push(src.into_future().unwrap());
        self
    }

    pub fn add_include<T: IntoFuture<Path>>(mut self, include: T) -> Gcc {
        self.includes.push(include.into_future().unwrap());
        self
    }

    pub fn add_lib<T: IntoFuture<Path>>(mut self, lib: T) -> Gcc {
        self.libs.push(lib.into_future().unwrap());
        self
    }

    pub fn add_external_lib<T: Str>(mut self, lib: T) -> Gcc {
        self.external_libs.push(lib.into_owned());
        self
    }

    pub fn add_libpath<T: IntoPath>(mut self, libpath: T) -> Gcc {
        self.libpaths.push(libpath.into_path());
        self
    }

    pub fn add_macro<T: Str>(mut self, macro: T) -> Gcc {
        self.macros.push(macro.into_owned());
        self
    }

    pub fn add_warning<T: Str>(mut self, warning: T) -> Gcc {
        self.warnings.push(warning.into_owned());
        self
    }

    pub fn set_debug(mut self, debug: bool) -> Gcc {
        self.debug = debug;
        self
    }

    pub fn set_optimize(mut self, optimize: bool) -> Gcc {
        self.optimize = optimize;
        self
    }

    pub fn set_profile(mut self, profile: bool) -> Gcc {
        self.profile = profile;
        self
    }

    pub fn add_flag<S: Str>(mut self, flag: S) -> Gcc {
        self.flags.push(flag.into_owned());
        self
    }

    pub fn run(self) -> Path {
        self.into_future().unwrap()
    }
}

impl IntoFuture<Path> for Gcc {
    fn into_future(self) -> Future<Path> {
        let Gcc {
            ctx,
            exe,
            dst,
            srcs,
            includes,
            libs,
            mut external_libs,
            mut libpaths,
            macros,
            warnings,
            debug,
            profile,
            optimize,
            flags
        } = self;

        assert!(!srcs.is_empty());

        let mut prep = ctx.prep("Call");
        let mut call = Call::new(exe.clone()).unwrap();

        let dst = match dst {
            Some(dst) => {
                call.push_str(~"-o");
                call.push_output_path(dst.clone());
                dst
            }
            None => { Path::new("") }
        };

        for include in includes.move_iter() {
            call.push_str(~"-I");
            call.push_input_path(include).unwrap();
        }

        // We need to extract the relative lib info from a lib path
        for lib in libs.move_iter() {
            prep.declare_input_path(lib.clone()).unwrap();

            libpaths.push(lib.dir_path());

            let name = lib.filename_str().unwrap();

            let prefix = "lib";
            let suffix = ".dylib";

            assert!(name.starts_with(prefix) && name.ends_with(suffix));

            external_libs.push(name.slice(prefix.len(), suffix.len()).to_owned());
        }

        for libpath in libpaths.move_iter() {
            call.push_str(~"-L");
            call.push_str(libpath.as_str().unwrap().to_owned());
        }

        for lib in external_libs.move_iter() {
            call.push_str(~"-l");
            call.push_str(lib);
        }

        if debug { call.push_str(~"-g"); }
        if optimize { call.push_str(~"-O2"); }
        if profile { call.push_str(~"-pg"); }

        for macro in macros.move_iter() {
            call.push_str(~"-D");
            call.push_str(macro);
        }

        for warning in warnings.move_iter() {
            call.push_str(~"-W");
            call.push_str(warning);
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

#[deriving(Clone)]
pub struct Ar {
    priv ctx: Context,
    priv exe: Path,
    priv dst: Path,
    priv srcs: ~[Path],
    priv flags: ~[~str],
}

impl Ar {
    pub fn new<T: IntoPath, U: IntoPath>(ctx: Context, exe: T, dst: U) -> Ar {
        Ar {
            ctx: ctx,
            exe: exe.into_path(),
            dst: dst.into_path(),
            srcs: ~[],
            flags: ~[~"-rc"],
        }
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
            dst,
            srcs,
            flags
        } = self;

        let mut call = Call::new(exe.clone()).unwrap();

        for flag in flags.move_iter() {
            call.push_str(flag);
        }

        call.push_input_path(dst.clone()).unwrap();

        let mut prep = ctx.prep("Call");
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
