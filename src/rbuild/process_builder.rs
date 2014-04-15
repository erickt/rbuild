use std::fmt::Show;
use std::io;
use std::io::{IoResult, MemWriter, Process, ProcessConfig};
use std::io::process::{ProcessExit, ProcessOutput};
use std::str;
use term::color::Color;

pub struct ProcessBuilder<'a> {
    config: ProcessConfig<'a>,
    color: Option<Color>,
    verbosity: uint,
    stdout_verbosity: Option<uint>,
    stderr_verbosity: Option<uint>,
    msgs: MemWriter,
    timeout: Option<uint>,
}

impl<'a> ProcessBuilder<'a> {
    pub fn new(program: &'a str, args: &'a [~str]) -> ProcessBuilder<'a> {
        let config = ProcessConfig {
            program: program,
            args: args,
            .. ProcessConfig::new()
        };

        ProcessBuilder {
            config: config,
            color: None,
            verbosity: 0,
            stdout_verbosity: None,
            stderr_verbosity: None,
            msgs: MemWriter::new(),
            timeout: None,
        }
    }

    pub fn color(mut self, color: Color) -> ProcessBuilder<'a> {
        self.color = Some(color);
        self
    }

    pub fn verbosity(mut self, verbosity: uint) -> ProcessBuilder<'a> {
        self.verbosity = verbosity;
        self
    }

    pub fn stdout_verbosity(mut self, verbosity: uint) -> ProcessBuilder<'a> {
        self.stdout_verbosity = Some(verbosity);
        self
    }

    pub fn stderr_verbosity(mut self, verbosity: uint) -> ProcessBuilder<'a> {
        self.stderr_verbosity = Some(verbosity);
        self
    }

    pub fn description<T: Show>(mut self, description: T) -> ProcessBuilder<'a> {
        (write!(&mut self.msgs, " * {:10}:", description)).unwrap();
        self
    }

    pub fn msg<T: Show>(mut self, msg: T) -> ProcessBuilder<'a> {
        (write!(&mut self.msgs, " {}", msg)).unwrap();
        self
    }

    pub fn msgs<T: Show, Iter: Iterator<T>>(mut self, mut msgs: Iter) -> ProcessBuilder<'a> {
        for msg in msgs {
            (write!(&mut self.msgs, " {}", msg)).unwrap();
        }
        self
    }

    pub fn run(self) -> IoResult<ProcessExit> {
        let out = try!(self.run_with_output());
        Ok(out.status)
    }

    pub fn run_with_output(self) -> IoResult<ProcessOutput> {
        let mut cmd = StrBuf::from_str(self.config.program);
        for arg in self.config.args.iter() {
            cmd.push_str(" ");
            cmd.push_str(*arg);
        }

        debug!("running {}", cmd);

        let mut stdout = io::stdout();

        let msgs = self.msgs.get_ref();
        if !msgs.is_empty() {
            try!(stdout.write(msgs));
            try!(stdout.write_str("\n"));
        }

        let mut process = try!(Process::configure(self.config));
        let output = process.wait_with_output();

        // If we errored out, log the error.
        if !output.status.success() {
            try!(stdout.write_str(" + "));
            try!(stdout.write_str(cmd.as_slice().trim_right()));
            try!(stdout.write_str("\n"));

            let out = output.output.as_slice();
            let out1 = str::from_utf8_lossy(out);
            let out2 = out1.as_slice().trim_right();
            if !out2.is_empty() {
                try!(stdout.write_str(out2));
                try!(stdout.write_str("\n"));
            }

            let err = output.error.as_slice();
            let err1 = str::from_utf8_lossy(err);
            let err2 = err1.as_slice().trim_right();
            if !err2.is_empty() {
                try!(stdout.write_str(err2));
                try!(stdout.write_str("\n"));
            }

            try!(stdout.flush());
        }
        Ok(output)
    }
}
