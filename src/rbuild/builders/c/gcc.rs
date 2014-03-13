use std::io;
use std::io::process::Process;
use std::io::fs;
use sync::Future;

use builders::ar::Ar;
use context::{Context, Call};
use into_path::IntoPath;
use into_future::IntoFuture;
use path_util;

pub static COMPILE_SUFFIX: &'static str = "o";
pub static LIB_PREFIX: &'static str = "lib";
pub static STATIC_LIB_SUFFIX: &'static str = "a";
pub static SHARED_LIB_SUFFIX: &'static str = "dylib";

#[deriving(Clone)]
pub struct StaticBuilder {
    priv gcc: Gcc,
    priv ar: Ar,
}

impl StaticBuilder {
    pub fn new(ctx: Context) -> StaticBuilder {
        StaticBuilder::new_with(ctx, "/usr/bin/gcc", "/usr/bin/ar")
    }

    pub fn new_with<T: Str, U: Str>(ctx: Context, gcc_exe: T, ar_exe: U) -> StaticBuilder {
        let gcc = Gcc::new(
            ctx.clone(),
            gcc_exe.into_owned(),
            LIB_PREFIX,
            STATIC_LIB_SUFFIX);

        let ar = Ar::new(ctx, ar_exe.into_owned());

        StaticBuilder {
            gcc: gcc,
            ar: ar,
        }
    }

    pub fn compile<T: IntoFuture<Path>>(&self, src: T) -> Gcc {
        let src = src.into_future().unwrap();
        let dst = src.with_extension(COMPILE_SUFFIX);

        self.gcc.clone()
            .set_dst(dst)
            .set_dst_suffix(COMPILE_SUFFIX)
            .add_src(src)
            .add_flag(~"-c")
    }

    pub fn link_lib<T: IntoPath>(&self, dst: T) -> Ar {
        self.ar.clone()
            .set_dst(dst)
            .set_dst_prefix(LIB_PREFIX)
            .set_dst_suffix(STATIC_LIB_SUFFIX)
    }

    pub fn link_exe<T: IntoPath>(&self, dst: T) -> Gcc {
        self.gcc.clone()
            .set_dst(dst)
    }

    pub fn add_include<T: IntoFuture<Path>>(self, include: T) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.add_include(include), ar: ar }
    }

    pub fn add_lib<T: IntoFuture<Path>>(self, lib: T) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.add_lib(lib), ar: ar }
    }

    pub fn add_external_lib<T: Str>(self, lib: T) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.add_external_lib(lib), ar: ar }
    }

    pub fn add_libpath<T: IntoPath>(self, libpath: T) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.add_libpath(libpath), ar: ar }
    }

    pub fn add_macro<T: Str>(self, macro: T) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.add_macro(macro), ar: ar }
    }

    pub fn add_warning<T: Str>(self, warning: T) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.add_warning(warning), ar: ar }
    }

    pub fn set_debug(self, debug: bool) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.set_debug(debug), ar: ar }
    }

    pub fn set_optimize(self, optimize: bool) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.set_optimize(optimize), ar: ar }
    }

    pub fn set_profile(self, profile: bool) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.set_profile(profile), ar: ar }
    }

    pub fn add_flag<S: Str>(self, flag: S) -> StaticBuilder {
        let StaticBuilder { gcc, ar } = self;
        StaticBuilder { gcc: gcc.add_flag(flag), ar: ar }
    }
}

#[deriving(Clone)]
pub struct SharedBuilder {
    priv gcc: Gcc,
}

impl SharedBuilder {
    pub fn new(ctx: Context) -> SharedBuilder {
        SharedBuilder::new_with(ctx, "/usr/bin/gcc")
    }

    pub fn new_with<T: Str>(ctx: Context, gcc_exe: T) -> SharedBuilder {
        let gcc = Gcc::new(
            ctx.clone(),
            gcc_exe.into_owned(),
            LIB_PREFIX,
            SHARED_LIB_SUFFIX);

        SharedBuilder {
            gcc: gcc,
        }
    }

    pub fn compile<T: IntoFuture<Path>>(&self, src: T) -> Gcc {
        let src = src.into_future().unwrap();
        let dst = src.with_extension(COMPILE_SUFFIX);

        self.gcc.clone()
            .set_dst(dst)
            .set_dst_suffix(COMPILE_SUFFIX)
            .add_src(src)
            .add_flag(~"-c")
            .add_flag(~"-fPIC")
    }

    pub fn link_lib<T: IntoPath>(&self, dst: T) -> Gcc {
        self.gcc.clone()
            .set_dst(dst)
            .set_dst_prefix(LIB_PREFIX)
            .set_dst_suffix(SHARED_LIB_SUFFIX)
            .add_flag(~"-fPIC")
            .add_flag(~"-dynamiclib")
    }

    pub fn link_exe<T: IntoPath>(&self, dst: T) -> Gcc {
        self.gcc.clone()
            .set_dst(dst)
    }

    pub fn add_include<T: IntoFuture<Path>>(self, include: T) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.add_include(include) }
    }

    pub fn add_lib<T: IntoFuture<Path>>(self, lib: T) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.add_lib(lib) }
    }

    pub fn add_external_lib<T: Str>(self, lib: T) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.add_external_lib(lib) }
    }

    pub fn add_libpath<T: IntoPath>(self, libpath: T) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.add_libpath(libpath) }
    }

    pub fn add_macro<T: Str>(self, macro: T) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.add_macro(macro) }
    }

    pub fn add_warning<T: Str>(self, warning: T) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.add_warning(warning) }
    }

    pub fn set_debug(self, debug: bool) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.set_debug(debug) }
    }

    pub fn set_optimize(self, optimize: bool) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.set_optimize(optimize) }
    }

    pub fn set_profile(self, profile: bool) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.set_profile(profile) }
    }

    pub fn add_flag<S: Str>(self, flag: S) -> SharedBuilder {
        let SharedBuilder { gcc } = self;
        SharedBuilder { gcc: gcc.add_flag(flag) }
    }
}

#[deriving(Clone)]
pub struct Gcc {
    priv ctx: Context,
    priv exe: Path,
    priv dst_prefix: Option<&'static str>,
    priv dst_suffix: Option<&'static str>,
    priv dst: Option<Path>,
    priv srcs: ~[Path],
    priv includes: ~[Path],
    priv lib_prefix: &'static str,
    priv lib_suffix: &'static str,
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
    pub fn new<T: IntoPath>(
        ctx: Context,
        exe: T,
        lib_prefix: &'static str,
        lib_suffix: &'static str
    ) -> Gcc {
        Gcc {
            ctx: ctx,
            exe: exe.into_path(),
            dst_prefix: None,
            dst_suffix: None,
            dst: None,
            srcs: ~[],
            includes: ~[],
            lib_prefix: lib_prefix,
            lib_suffix: lib_suffix,
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

    pub fn set_dst_prefix(mut self, dst_prefix: &'static str) -> Gcc {
        self.dst_prefix = Some(dst_prefix);
        self
    }

    pub fn set_dst_suffix(mut self, dst_suffix: &'static str) -> Gcc {
        self.dst_suffix = Some(dst_suffix);
        self
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
            dst_prefix,
            dst_suffix,
            lib_prefix,
            lib_suffix,
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
            Some(mut dst) => {
                dst = path_util::add_prefix_suffix(dst, dst_prefix, dst_suffix);

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

            assert!(name.starts_with(lib_prefix) && name.ends_with(lib_suffix));

            external_libs.push(name.slice(lib_prefix.len(), name.len() - (lib_suffix.len() + 1)).to_owned());
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
