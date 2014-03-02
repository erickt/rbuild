use std::io::{IoResult, File};
use std::hash;
use std::num::ToStrRadix;
use collections::TreeMap;

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
        freshness.insert(~"paths", paths_are_fresh);
        freshness.insert(~"value", value_is_fresh);

        let ctx = ::workcache::Context::new_with_freshness(db, logger, cfg, freshness);

        Context { ctx : ctx }
    }

    pub fn prep<'a>(&'a self, fn_name: &'a str) -> ::workcache::Prep<'a> {
        self.ctx.prep(fn_name)
    }
}

/// Hashes the file contents along with the last-modified time
pub fn digest<'a, T: hash::Hash>(value: &T) -> ~str {
    println!("digest: {:?}", value);
    hash::hash(value).to_str_radix(16)
}

/// Hashes the file contents along with the last-modified time
pub fn digest_file(path: &Path) -> IoResult<~str> {
    let st = try!(path.stat());

    let mut file = try!(File::open(path));
    let bytes = try!(file.read_to_end());
    let digest = digest(&bytes);

    println!("digesting: {} {:?} {}", path.display(), st.modified, digest);

    Ok(digest)
}

/// Hashes only the last-modified time
pub fn digest_only_date(path: &Path) -> IoResult<~str> {
    let st = try!(path.stat());
    Ok(digest(&st.modified))
}

extern "C" fn path_is_fresh(path: &str, in_hash: &str) -> bool {
    let path = Path::new(path);

    println!("path_is_fresh: {} {}", path.display(), in_hash);

    path.exists() && in_hash == digest_file(&path).unwrap()
}

extern "C" fn paths_are_fresh(path: &str, in_hash: &str) -> bool {
    let path = Path::new(path);

    println!("paths_are_fresh: {} {}", path.display(), in_hash);

    path.exists() && in_hash == digest_file(&path).unwrap()
}


extern "C" fn value_is_fresh(path: &str, in_hash: &str) -> bool {
    let path = Path::new(path);

    println!("value_is_fresh: {} {}", path.display(), in_hash);

    in_hash == digest_file(&path).unwrap()
}
