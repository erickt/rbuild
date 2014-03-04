use std::io::IoResult;
use std::io::process::Process;
use std::vec;

use context::Context;

#[deriving(Clone)]
pub struct Compiler {
    ctx: Context,
    exe: Path,
    flags: ~[~str],
}

impl Compiler {
    pub fn new(ctx: Context, exe: Path) -> Compiler {
        Compiler {
            ctx: ctx,
            exe: exe,
            flags: ~[],
        }
    }

    pub fn build_object(&self, src: Path) -> Path {
        self.build_object_with(None, src, &[~"-c"])
    }

    pub fn build_object_with(&self, dst: Option<Path>, src: Path, flags: &[~str]) -> Path {
        let dst = dst.or_else(|| {
            Some(src.clone().with_extension("o"))
        });

        let call = Call {
            exe: self.exe.clone(),
            dst: dst,
            srcs: ~[src],
            flags: vec::append(self.flags.clone(), flags),
        };

        let mut prep = self.ctx.prep("Call");

        prep.declare_input_path("exe", &call.exe);
        prep.declare_input_paths("srcs", call.srcs);

        prep.exec(proc(exec) {
            call.run().unwrap();
            let dst = call.dst.unwrap();
            exec.discover_output_path("dst", &dst);
            dst
        })
    }

    /*
    pub fn build_objects(&self, srcs: &[Path]) -> ~[Call] {
        srcs.iter().map(|src| {
            self.build_object(src.clone())
        }).collect()
    }

    pub fn build_exe(&self, dst: Path, srcs: ~[Path]) -> Call {
        let call = Call {
            exe: self.exe.clone(),
            dst: Some(dst),
            srcs: srcs,
            flags: self.flags.clone(),
        };

        Call {
            ctx: self.ctx.clone(),
            gcc: gcc,
        }
    }
    */
}

#[deriving(Clone, Hash, Encodable)]
pub struct Call {
    priv exe: Path,
    priv dst: Option<Path>,
    priv srcs: ~[Path],
    priv flags: ~[~str],
}

impl Call {
    pub fn set_dst(mut self, dst: Path) -> Call {
        self.dst = Some(dst);
        self
    }

    pub fn add_src(mut self, src: Path) -> Call {
        self.srcs.push(src);
        self
    }

    pub fn add_flag<S: Str>(mut self, flag: S) -> Call {
        self.flags.push(flag.as_slice().to_owned());
        self
    }

    pub fn add_flags<S: Str>(mut self, flags: &[S]) -> Call {
        for flag in flags.iter() {
            let flag = flag.as_slice().to_owned();
            self.flags.push(flag);
        }

        self
    }

    pub fn run(&self) -> IoResult<()> {
        let exe = self.exe.as_str().unwrap().to_owned();

        let mut cmd = ~[];

        match self.dst {
            Some(ref dst) => {
                cmd.push_all([~"-o", dst.as_str().unwrap().to_owned()]);
            }
            None => {}
        }

        for flag in self.flags.iter() {
            cmd.push(flag.as_slice().to_owned());
        }

        for src in self.srcs.iter() {
            cmd.push(src.as_str().unwrap().to_owned());
        }

        print!("{}:", self.exe.display());
        for src in self.srcs.iter() {
            print!(" {}", src.display());
        }

        match self.dst {
            Some(ref dst) => {
                println!(" -> {}", dst.display());
            }
            None => {
                println!("");
            }
        }

        let status = try!(Process::status(exe, cmd));

        if !status.success() {
            fail!("command failed: {:?}", cmd);
        }

        Ok(())
    }
}

/*
pub struct Call {
    priv ctx: Context,
    priv gcc: Gcc,
}

impl Call {
    pub fn add_flags<S: Str>(&mut self, flags: &[S]) {
        self.gcc.add_flags(flags)
    }

    pub fn run(self) -> Path {
        let Call { ctx, gcc } = self;

        let mut prep = ctx.prep("Call");

        prep.declare_input("value", "gcc", &gcc);
        prep.declare_input_path("exe", &gcc.exe);
        prep.declare_input_paths("srcs", gcc.srcs);

        prep.exec(proc(exec) {
            gcc.run().unwrap();
            let dst = gcc.dst.unwrap();
            exec.discover_output_path("dst", &dst);
            dst
        })
    }
}
*/

/*
struct BuildObjectsExec {
    ctx: Context,
    opts: BuildObjectOptions,
}

struct BuildObjectsOptions {
    exe: Path,
    srcs: ~[Path],
    flags: ~[~str],
}

impl Call {
    pub fn run(self) -> Path {
        let Call { ctx, opts } = self;

        let mut prep = ctx.prep("BuildObjectExec");

        prep.declare_input_path(&opts.exe);

        for src in opts.srcs {
            prep.declare_input_path(&src);
        }

        prep.declare_input("flags", "flags", &opts.flags);

        let (port,chan) = comm::stream();
        chan.send(opts);

        do prep.exec |exec| {
            let GccOptions { exe, dst, src, flags } = port.recv();

            let mut cmd = ~[exe.to_str()];

            let dst = match dst {
                Some(dst) => dst,
                None => Path(src.to_str() + ".o"),
            };

            cmd.push_all([~"-o", dst.to_str()]);

            for flag in flags.move_iter() {
                cmd.push(flag);
            }

            cmd.push(src.to_str());

            println!("{}: {} -> {}", exe.to_str(), src.to_str(), dst.to_str());

            let status = run::process_status(*cmd.head(), cmd.tail());

            if status == 0 {
                exec.declare_output_path(&dst);
                dst
            } else {
                fail!("command failed: %?", cmd);
            }
        }
    }
}

struct BuildExeExec {
    ctx: Context,
    opts: BuildExeOptions,
}

struct BuildExeOptions {
    exe: Path,
    dst: Path,
    srcs: ~[Path],
    flags: ~[~str],
}

impl BuildExeExec {
    pub fn run(self) -> ~str {
        let BuildExeExec { ctx, opts } = self;

        let mut prep = ctx.prep("BuildExeExec");

        prep.declare_input_path(&opts.exe);
        prep.declare_input_path(&opts.dst);

        for src in opts.srcs.iter() {
            prep.declare_input_path(src);
        }

        prep.declare_input("flags", "flags", &opts.flags);

        let (port,chan) = comm::stream();
        chan.send(opts);

        do prep.exec |exec| {
            let BuildExeOptions { exe, dst, srcs, flags } = port.recv();

            let mut cmd = ~[exe.to_str()];

            cmd.push_all([~"-o", dst.to_str()]);

            for flag in flags.move_iter() {
                cmd.push(flag);
            }

            for src in srcs.iter() {
                cmd.push(src.to_str());
            }

            println!("{}: {} -> {}",
                exe.to_str(),
                srcs.map(|src| src.to_str()).connect(" "),
                dst.to_str());

            let status = run::process_status(*cmd.head(), cmd.tail());

            if status == 0 {
                exec.declare_output_path(&dst);
                dst.to_str()
            } else {
                fail!("command failed: %?", cmd);
            }
        }
    }
}
*/
