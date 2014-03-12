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

    pub fn compile<T: IntoFuture<Path>>(&self, src: T) -> Compile {
        let gcc = Gcc::new(self.ctx.clone(), self.exe.clone())
            .add_src(src)
            .add_flag(~"-c");

        Compile { gcc: gcc }
    }

    pub fn link_exe<
        'a,
        Dst: IntoPath,
        Src: IntoFuture<Path>
    >(&self, dst: Dst, srcs: ~[Src]) -> LinkExe {
        let gcc = Gcc::new(self.ctx.clone(), self.exe.clone())
            .set_dst(dst)
            .add_srcs(srcs);

        LinkExe { gcc: gcc }
    }
}

pub struct Compile {
    priv gcc: Gcc,
}

impl Compile {
    pub fn set_dst<T: IntoPath>(self, dst: T) -> Compile {
        let Compile { gcc } = self;
        Compile { gcc: gcc.set_dst(dst) }
    }

    pub fn add_src<T: IntoFuture<Path>>(self, src: T) -> Compile {
        let Compile { gcc } = self;
        Compile { gcc: gcc.add_src(src) }
    }

    pub fn add_srcs<T: IntoFuture<Path>>(self, srcs: ~[T]) -> Compile {
        let Compile { gcc } = self;
        Compile { gcc: gcc.add_srcs(srcs) }
    }

    pub fn add_flag<S: Str>(self, flag: S) -> Compile {
        let Compile { gcc } = self;
        Compile { gcc: gcc.add_flag(flag) }
    }

    pub fn run(self) -> Future<Path> {
        let Compile { mut gcc } = self;

        assert!(!gcc.srcs.is_empty());

        let dst = match gcc.dst.take() {
            Some(dst) => dst,
            None => gcc.srcs[0].with_extension("o"),
        };

        let dst = gcc.ctx.root.join(dst);
        gcc.dst = Some(dst.clone());

        Future::from_fn(proc() {
            gcc.run().unwrap();
            dst
        })
    }
}

pub struct LinkExe {
    priv gcc: Gcc,
}

impl LinkExe {
    pub fn add_src<T: IntoFuture<Path>>(self, src: T) -> LinkExe {
        let LinkExe { gcc } = self;
        LinkExe { gcc: gcc.add_src(src) }
    }

    pub fn add_srcs<T: IntoFuture<Path>>(self, srcs: ~[T]) -> LinkExe {
        let LinkExe { gcc } = self;
        LinkExe { gcc: gcc.add_srcs(srcs) }
    }


    pub fn add_flag<S: Str>(self, flag: S) -> LinkExe {
        let LinkExe { gcc } = self;
        LinkExe { gcc: gcc.add_flag(flag) }
    }

    pub fn run(self) -> Future<Path> {
        let LinkExe { mut gcc } = self;

        assert!(!gcc.srcs.is_empty());

        let dst = gcc.dst.take_unwrap();
        let dst = gcc.ctx.root.join(dst);
        gcc.dst = Some(dst.clone());

        Future::from_fn(proc() {
            gcc.run().unwrap();
            dst
        })
    }
}

struct Gcc {
    ctx: Context,
    exe: Path,
    dst: Option<Path>,
    srcs: ~[Path],
    flags: ~[~str],
}

impl Gcc {
    pub fn new(ctx: Context, exe: Path) -> Gcc {
        Gcc {
            ctx: ctx,
            exe: exe,
            dst: None,
            srcs: ~[],
            flags: ~[],
        }
    }

    pub fn set_dst<T: IntoPath>(mut self, dst: T) -> Gcc {
        self.dst = Some(dst.into_path());
        self
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

    pub fn add_flag<S: Str>(mut self, flag: S) -> Gcc {
        self.flags.push(flag.as_slice().to_owned());
        self
    }

    fn run(self) -> Future<()> {
        let Gcc { ctx, exe, dst, srcs, flags } = self;

        assert!(!srcs.is_empty());

        let mut call = Call::new(exe.clone());

        match dst {
            Some(ref dst) => {
                call.push_str(~"-o");
                call.push_output_path(dst.clone());
            }
            None => { }
        }

        for flag in flags.move_iter() {
            call.push_str(flag);
        }

        for src in srcs.iter() {
            call.push_input_path(src.clone());
        }

        let mut prep = ctx.prep("Call");
        prep.declare_call(&call);

        prep.exec(proc(_exec) {
            let (prog, args) = call.cmd();

            print!("{}:", exe.display());

            for src in srcs.iter() {
                print!(" {}", src.display());
            }

            match dst {
                Some(ref dst) => {
                    // Make sure the parent directories exist.
                    fs::mkdir_recursive(&dst.dir_path(), io::UserDir).unwrap();

                    println!(" -> {}", dst.display());
                }
                None => {
                    println!("");
                }
            }

            println!("{} {}", prog, args);

            let mut process = Process::new(prog, args).unwrap();
            let status = process.wait();

            if !status.success() {
                fail!("command failed");
            }
        })
    }
}
