use std::io::{IoResult, File};
use std::io::MemWriter;
use std::str;
use std::hash;
use std::num::ToStrRadix;
use collections::TreeMap;
use serialize::json;
use serialize::{Encodable, Decodable};
use sync::Future;

use workcache;

#[deriving(Clone)]
pub struct Context {
    ctx: ::workcache::Context,
}

impl Context {
    pub fn new(path: Path) -> Context {
        let db = ::workcache::Database::new(path);
        let logger = ::workcache::Logger::new();
        let cfg = TreeMap::new();

        let mut freshness = TreeMap::new();
        freshness.insert(~"call", call_is_fresh);

        let ctx = workcache::Context::new_with_freshness(db, logger, cfg, freshness);

        Context { ctx : ctx }
    }

    pub fn prep<T: str::IntoMaybeOwned<'static>>(&self, fn_name: T) -> Prep {
        Prep { prep: self.ctx.prep(fn_name) }
    }

    pub fn prep_call<T: str::IntoMaybeOwned<'static>>(&self, fn_name: T, call: &Call) -> Prep {
        let mut prep = self.prep(fn_name);
        prep.declare_call(call);
        prep
    }
}

pub struct Prep {
    priv prep: workcache::Prep,
}

impl Prep {
    pub fn declare_input<
        'a,
        T: Encodable<json::Encoder<'a>>
    >(&mut self, kind: &str, name: &str, value: &T) {
        self.prep.declare_input(kind, name, json_encode(value))
    }

    pub fn declare_call(&mut self, call: &Call) {
        self.declare_input("call", "", call)
    }

    pub fn exec<
        'a,
        T: Send + Encodable<json::Encoder<'a>> + Decodable<json::Decoder>
    >(self, blk: proc(&mut Exec) -> T) -> Future<T> {
        self.prep.exec(proc(exec) {
            let mut exec = Exec { exec: exec };
            blk(&mut exec)
        })
    }
}

pub struct Exec<'a> {
    priv exec: &'a mut workcache::Exec,
}

impl<'a> Exec<'a> {
    pub fn discover_input<
        T: Encodable<json::Encoder<'a>>
    >(&mut self, kind: &str, name: &str, value: &T) {
        self.exec.discover_input(kind, name, json_encode(value))
    }

    pub fn discover_input_path(&mut self, name: &str, path: Path) {
        self.discover_input("path", name, &InputPath::new(path))
    }

    pub fn discover_output<
        T: Encodable<json::Encoder<'a>>
    >(&mut self, kind: &str, name: &str, value: &T) {
        self.exec.discover_output(kind, name, json_encode(value))
    }

    pub fn discover_output_path(&mut self, name: &str, path: Path) {
        self.discover_output("path", name, &path)
    }
}

/// Hashes the path contents
fn digest_path(path: &Path) -> IoResult<~str> {
    let mut file = try!(File::open(path));
    let bytes = try!(file.read_to_end());
    let digest = hash::hash(&bytes);

    println!("digesting: {} {}", path.display(), digest);

    Ok(digest.to_str_radix(16))
}

#[deriving(Encodable, Decodable)]
struct InputPath {
    path: Path,
    digest: ~str,
    modified: u64,
}

impl InputPath {
    fn new(path: Path) -> InputPath {
        let digest = digest_path(&path).unwrap();
        let st = path.stat().unwrap();

        InputPath {
            path: path,
            digest: digest,
            modified: st.modified,
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    fn is_fresh(&self) -> bool {
        self.exists() && self.digest == digest_path(&self.path).unwrap()
    }
}

#[deriving(Encodable, Decodable)]
pub struct Call {
    priv prog: CallArg,
    priv args: ~[CallArg],
}

impl Call {
    pub fn new(prog: Path) -> Call {
        Call {
            prog: InputPath(InputPath::new(prog)),
            args: ~[],
        }
    }

    pub fn push_str(&mut self, value: ~str) {
        self.args.push(Str(value))
    }

    pub fn push_input_path(&mut self, value: Path) {
        self.args.push(InputPath(InputPath::new(value)))
    }

    pub fn push_output_path(&mut self, value: Path) {
        self.args.push(OutputPath(value))
    }

    fn is_fresh(&self) -> bool {
        self.args.iter().all(|arg| arg.is_fresh())
    }

    pub fn cmd(&self) -> (~str, ~[~str]) {
        fn f(arg: &CallArg) -> ~str {
            match *arg {
                Str(ref s) => s.clone(),
                InputPath(ref p) => p.path.as_str().unwrap().to_owned(),
                OutputPath(ref p) => p.as_str().unwrap().to_owned(),
            }
        }

        let prog = f(&self.prog);
        let args = self.args.iter().map(f).collect();

        (prog, args)
    }
}

#[deriving(Encodable, Decodable)]
enum CallArg {
    Str(~str),
    InputPath(InputPath),
    OutputPath(Path),
}

impl CallArg {
    fn is_fresh(&self) -> bool {
        match *self {
            Str(_) => true,
            InputPath(ref p) => p.is_fresh(),
            OutputPath(ref p) => p.exists(),
        }
    }
}

fn json_encode<'a, T:Encodable<json::Encoder<'a>>>(t: &T) -> ~str {
    let mut writer = MemWriter::new();
    let mut encoder = json::Encoder::new(&mut writer);
    t.encode(&mut encoder);
    str::from_utf8_owned(writer.unwrap()).unwrap()
}

fn json_decode<T:Decodable<json::Decoder>>(s: &str) -> T {
    let j = json::from_str(s).unwrap();
    let mut decoder = json::Decoder::new(j);
    Decodable::decode(&mut decoder)
}

fn call_is_fresh(_name: &str, value: &str) -> bool {
    let call: Call = json_decode(value);

    call.is_fresh()
}
