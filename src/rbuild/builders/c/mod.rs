use builders::ar::Ar;
use context::Context;
use into_path::IntoPath;
use into_future::IntoFuture;

use self::gcc::Gcc;

pub mod gcc;

#[deriving(Clone)]
pub struct StaticBuilder {
    gcc: Gcc,
    ar: Ar,
}

pub static COMPILE_PREFIX: &'static str = "";
pub static COMPILE_SUFFIX: &'static str = "o";

pub static LIB_PREFIX: &'static str = "lib";
pub static STATIC_LIB_SUFFIX: &'static str = "a";

#[cfg(target_os = "linux")]
pub static SHARED_LIB_SUFFIX: &'static str = "so";

#[cfg(target_os = "macos")]
pub static SHARED_LIB_SUFFIX: &'static str = "dylib";

impl StaticBuilder {
    pub fn new(ctx: Context) -> StaticBuilder {
        StaticBuilder::new_with(
            Gcc::new(ctx.clone(), LIB_PREFIX, STATIC_LIB_SUFFIX),
            Ar::new(ctx.clone()))
    }

    pub fn new_with(gcc: Gcc, ar: Ar) -> StaticBuilder {
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
    gcc: Gcc,
}

impl SharedBuilder {
    pub fn new(ctx: Context) -> SharedBuilder {
        SharedBuilder::new_with(Gcc::new(ctx, LIB_PREFIX, SHARED_LIB_SUFFIX))
    }

    pub fn new_with(gcc: Gcc) -> SharedBuilder {
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
