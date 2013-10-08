use std::run;
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

    pub fn build_object(&self, src: Path) -> GccExec {
        let dst_filename: ~str = match src.filename() {
            Some(src_filename) => {
                [&"", src_filename, &".o"].concat()
            }
            None => fail!(),
        };
        let dst = Some(src.dir_path().with_filename(dst_filename));

        let gcc = Gcc {
            exe: self.exe.clone(),
            dst: dst,
            srcs: ~[src],
            flags: vec::append(self.flags.clone(), [~"-c"]),
        };

        GccExec {
            ctx: self.ctx.clone(),
            gcc: gcc,
        }
    }

    /*
    pub fn build_object_with(&self, dst: Option<Path>, src: Path, opts: BuildObjectOptions) -> BuildObjectExec {
        BuildObjectExec {
            ctx: self.ctx.clone(),
            opts: BuildObjectOptions {
                exe: self.exe.clone(),
                dst: None,
                src: src,
                flags: vec::append(self.flags.clone(), [~"-c"]),
            }
        }
    }

    pub fn build_objects(&self, src: &[Path]) -> BuildObjectsExec {
        BuildObjectExec {
            ctx: self.ctx.clone(),
            opts: BuildObjectOptions {
                exe: self.exe.clone(),
                dst: None,
                srcs: srcs,
                flags: vec::append(self.flags.clone(), [~"-c"]),
            }
        }
    }
    */

    pub fn build_exe(&self, dst: Path, srcs: ~[Path]) -> GccExec {
        let gcc = Gcc {
            exe: self.exe.clone(),
            dst: Some(dst),
            srcs: srcs,
            flags: self.flags.clone(),
        };

        GccExec {
            ctx: self.ctx.clone(),
            gcc: gcc,
        }
    }
}

#[deriving(Clone, Decodable, Encodable)]
struct Gcc {
    exe: Path,
    dst: Option<Path>,
    srcs: ~[Path],
    flags: ~[~str],
}

impl Gcc {
    pub fn add_flags<S: Str>(&mut self, flags: &[S]) {
        for flag in flags.iter() {
            let flag = flag.as_slice().to_owned();
            self.flags.push(flag);
        }
    }

    pub fn run(&self) {
        let mut cmd = ~[self.exe.to_str()];

        match self.dst {
            Some(ref dst) => {
                cmd.push_all([~"-o", dst.to_str()]);
            }
            None => {}
        }

        for flag in self.flags.iter() {
            cmd.push(flag.as_slice().to_owned());
        }

        for src in self.srcs.iter() {
            cmd.push(src.to_str());
        }

        match self.dst {
            Some(ref dst) => {
                println!("{}: {} -> {}",
                    self.exe.to_str(),
                    self.srcs.map(|src| src.to_str()).connect(" "),
                    dst.to_str()
                );
            }
            None => {
                println!("{}: {}",
                    self.exe.to_str(),
                    self.srcs.map(|src| src.to_str()).connect(" ")
                );
            }
        }

        let status = run::process_status(*cmd.head(), cmd.tail());

        if status != 0 {
            fail!("command failed: %?", cmd);
        }
    }
}


struct GccExec {
    ctx: Context,
    gcc: Gcc,
}

impl GccExec {
    pub fn add_flags<S: Str>(&mut self, flags: &[S]) {
        self.gcc.add_flags(flags)
    }

    pub fn run(self) -> Path {
        let GccExec { ctx, gcc } = self;

        let mut prep = ctx.prep("GccExec");

        prep.declare_input("encodable", "gcc", &gcc);
        prep.declare_input_path(&gcc.exe);
        for src in gcc.srcs.iter() {
            prep.declare_input_path(src);
        }

        do prep.exec_with(gcc) |exec, gcc| {
            gcc.run();
            let dst = gcc.dst.unwrap();
            exec.declare_output_path(&dst);
            dst
        }
    }
}

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

impl GccExec {
    pub fn run(self) -> Path {
        let GccExec { ctx, opts } = self;

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
