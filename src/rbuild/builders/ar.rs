use std::io;
use std::io::process::Process;
use std::io::fs;
use sync::Future;

use context::{Context, Call};
use into_path::IntoPath;
use into_future::IntoFuture;

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
