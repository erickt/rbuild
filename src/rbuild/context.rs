use std::io::{IoResult, File};
use std::io::MemWriter;
use std::str;
use std::hash;
use std::num::ToStrRadix;
use collections::TreeMap;
use serialize::json;
use serialize::{Encodable, Decodable};

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
        freshness.insert(~"path", path_is_fresh);
        /*
        freshness.insert(~"paths", paths_are_fresh);
        freshness.insert(~"value", value_is_fresh);
        */

        let ctx = workcache::Context::new_with_freshness(db, logger, cfg, freshness);

        Context { ctx : ctx }
    }

    pub fn prep<'a>(&'a self, fn_name: &'a str) -> Prep<'a> {
        Prep { prep: self.ctx.prep(fn_name) }
    }
}

pub struct Prep<'a> {
    priv prep: workcache::Prep<'a>,
}

impl<'a> Prep<'a> {
    pub fn declare_input<
        T: Encodable<json::Encoder<'a>>
    >(&mut self, kind: &str, name: &str, value: &T) {
        self.prep.declare_input(kind, name, json_encode(value))
    }

    pub fn declare_input_path(&mut self, name: &str, path: &Path) {
        self.declare_input("path", name, &CachedPath::new(path))
    }

    pub fn declare_input_paths(&mut self, name: &str, paths: &[Path]) {
        for path in paths.iter() {
            self.declare_input_path(name, path)
        }
    }

    pub fn exec<
        'a,
        T: Send + Encodable<json::Encoder<'a>> + Decodable<json::Decoder>
    >(&self, blk: proc(&mut Exec) -> T) -> T {
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

    pub fn discover_input_path(&mut self, name: &str, path: &Path) {
        self.discover_input("path", name, &CachedPath::new(path))
    }

    pub fn discover_input_paths(&mut self, name: &str, paths: &[Path]) {
        for path in paths.iter() {
            self.discover_input_path(name, path)
        }
    }

    pub fn discover_output<
        T: Encodable<json::Encoder<'a>>
    >(&mut self, kind: &str, name: &str, value: &T) {
        self.exec.discover_output(kind, name, json_encode(value))
    }

    pub fn discover_output_path(&mut self, name: &str, path: &Path) {
        self.discover_output("path", name, &CachedPath::new(path))
    }

    pub fn discover_output_paths(&mut self, name: &str, paths: &[Path]) {
        for path in paths.iter() {
            self.discover_output_path(name, path)
        }
    }
}

/// Hashes the path contents
pub fn digest_path(path: &Path) -> IoResult<~str> {
    let mut file = try!(File::open(path));
    let bytes = try!(file.read_to_end());
    let digest = hash::hash(&bytes);

    println!("digesting: {} {}", path.display(), digest);

    Ok(digest.to_str_radix(16))
}

/// Hashes only the last-modified time
pub fn digest_only_date(path: &Path) -> IoResult<~str> {
    let st = try!(path.stat());
    let digest = hash::hash(&st.modified);

    Ok(digest.to_str_radix(16))
}

#[deriving(Encodable, Decodable)]
struct CachedPath {
    path: Path,
    digest: ~str,
    modified: u64,
}

impl CachedPath {
    fn new(path: &Path) -> CachedPath {
        let digest = digest_path(path).unwrap();
        let st = path.stat().unwrap();

        CachedPath {
            path: path.clone(),
            digest: digest,
            modified: st.modified,
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

fn path_is_fresh(_name: &str, value: &str) -> bool {
    let value: CachedPath = json_decode(value);

    value.path.exists() && value.digest == digest_path(&value.path).unwrap()
}

/*
extern "C" fn paths_are_fresh(path: &str, in_hash: u64) -> bool {
    let path = Path::new(path);

    println!("paths_are_fresh: {} {}", path.display(), in_hash);

    path.exists() && in_hash == digest_path(&path).unwrap()
}


extern "C" fn value_is_fresh(path: &str, in_hash: u64) -> bool {
    let path = Path::new(path);

    println!("value_is_fresh: {} {}", path.display(), in_hash);

    in_hash == digest_path(&path).unwrap()
}
*/
