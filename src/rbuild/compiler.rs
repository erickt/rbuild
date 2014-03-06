use std::io::process::Process;

use context::{Context, Call};

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

    pub fn compile(&self, src: Path) -> Compile {
        Compile {
            gcc: Gcc {
                ctx: self.ctx.clone(),
                exe: self.exe.clone(),
                dst: None,
                srcs: ~[src],
                flags: ~[~"-c"],
            }
        }
    }

    pub fn link_exe(&self, dst: Path, srcs: ~[Path]) -> LinkExe {
        LinkExe {
            gcc: Gcc {
                ctx: self.ctx.clone(),
                exe: self.exe.clone(),
                dst: Some(dst),
                srcs: srcs,
                flags: ~[],
            }
        }
    }
}

pub struct Compile {
    priv gcc: Gcc,
}

impl Compile {
    pub fn set_dst(self, dst: Path) -> Compile {
        let Compile { gcc } = self;
        Compile { gcc: gcc.set_dst(dst) }
    }

    pub fn add_src(self, src: Path) -> Compile {
        let Compile { gcc } = self;
        Compile { gcc: gcc.add_src(src) }
    }

    pub fn add_flag<S: Str>(self, flag: S) -> Compile {
        let Compile { gcc } = self;
        Compile { gcc: gcc.add_flag(flag) }
    }

    pub fn run(self) -> Path {
        let Compile { mut gcc } = self;

        assert!(!gcc.srcs.is_empty());

        let dst = match gcc.dst {
            Some(ref dst) => dst.clone(),
            None => {
                let dst = gcc.srcs[0].with_extension("o");
                gcc.dst = Some(dst.clone());
                dst
            }
        };

        gcc.run();

        dst
    }
}

pub struct LinkExe {
    priv gcc: Gcc,
}

impl LinkExe {
    pub fn add_src(self, src: Path) -> LinkExe {
        let LinkExe { gcc } = self;
        LinkExe { gcc: gcc.add_src(src) }
    }

    pub fn add_flag<S: Str>(self, flag: S) -> LinkExe {
        let LinkExe { gcc } = self;
        LinkExe { gcc: gcc.add_flag(flag) }
    }

    pub fn run(self) -> Path {
        let LinkExe { gcc } = self;

        assert!(!gcc.srcs.is_empty());

        let dst = match gcc.dst {
            Some(ref dst) => dst.clone(),
            None => { fail!("expected dst") }
        };

        gcc.run();

        dst
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
    pub fn set_dst(mut self, dst: Path) -> Gcc {
        self.dst = Some(dst);
        self
    }

    pub fn add_src(mut self, src: Path) -> Gcc {
        self.srcs.push(src);
        self
    }

    pub fn add_flag<S: Str>(mut self, flag: S) -> Gcc {
        self.flags.push(flag.as_slice().to_owned());
        self
    }

    fn run(self) {
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
                Some(ref dst) => { println!(" -> {}", dst.display()); }
                None => { println!(""); }
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
