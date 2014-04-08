use std::io;
use std::io::fs;
use sync::Future;

use context::{Context, Call};
use into_path::IntoPath;
use into_future::IntoFuture;
use path_util;

pub static EXES: &'static [&'static str] = &'static ["gcc", "cc"];

#[deriving(Clone)]
pub struct Gcc {
    ctx: Context,
    exe: Path,
    dst_prefix: Option<&'static str>,
    dst_suffix: Option<&'static str>,
    dst: Option<Path>,
    srcs: Vec<Path>,
    includes: Vec<Path>,
    lib_prefix: &'static str,
    lib_suffix: &'static str,
    libs: Vec<Path>,
    external_libs: Vec<~str>,
    libpaths: Vec<Path>,
    macros: Vec<~str>,
    warnings: Vec<~str>,
    debug: bool,
    profile: bool,
    optimize: bool,
    flags: Vec<~str>,
}

impl Gcc {
    pub fn new(ctx: Context, lib_prefix: &'static str, lib_suffix: &'static str) -> Gcc {
        let exe = path_util::find_program(ctx.clone(), EXES);

        Gcc::new_with(ctx, exe, lib_prefix, lib_suffix)
    }

    pub fn new_with<T: IntoFuture<Path>>(
        ctx: Context,
        exe: T,
        lib_prefix: &'static str,
        lib_suffix: &'static str
    ) -> Gcc {
        Gcc {
            ctx: ctx,
            exe: exe.into_future().unwrap(),
            dst_prefix: None,
            dst_suffix: None,
            dst: None,
            srcs: Vec::new(),
            includes: Vec::new(),
            lib_prefix: lib_prefix,
            lib_suffix: lib_suffix,
            libs: Vec::new(),
            external_libs: Vec::new(),
            libpaths: Vec::new(),
            macros: Vec::new(),
            warnings: Vec::new(),
            debug: false,
            profile: false,
            optimize: false,
            flags: Vec::new(),
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
