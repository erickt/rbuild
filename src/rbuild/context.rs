use std::io;
use std::os;
use extra::digest::Digest;
use extra::json;
use extra::serialize::Encodable;
use extra::sha1::Sha1;
use extra::treemap::TreeMap;

#[deriving(Clone)]
pub struct Context {
    ctx: ::workcache::Context,
}

/// Hashes the file contents along with the last-modified time
pub fn digest_encodable<T: Encodable<json::Encoder>>(value: &T) -> ~str {
    println!("digest_encodable: {:?}", value);

    let s = do io::with_str_writer |wr| {
        let mut encoder = json::Encoder(wr);
        value.encode(&mut encoder)
    };

    let mut sha = Sha1::new();
    sha.input_str(s);
    sha.result_str()
}

/// Hashes the file contents along with the last-modified time
pub fn digest_file(path: &Path) -> ~str {
    let st = match path.stat() {
        Some(st) => st,
        None => { fail!("missing file"); }
    };

    let digest = match io::read_whole_file(path) {
        Ok(bytes) => {
            let mut sha = Sha1::new();
            sha.input(bytes);
            sha.result_str()
        }
        Err(e) => { fail!("error reading file: %?", e) }
    };

    println!("digesting: {} {:?} {}", path.to_str(), st.st_mtime, digest);

    digest
}

/*
/// Hashes only the last-modified time
pub fn digest_only_date(path: &Path) -> ~str {
    use cond = conditions::bad_stat::cond;

    let mut sha = ~Sha1::new();
    let st = match path.stat() {
                Some(st) => st,
                None => cond.raise((path.clone(), fmt!("Couldn't get file access time")))
    };
    (*sha).input_str(st.st_mtime.to_str());
    (*sha).result_str()
}
*/

fn file_is_fresh(path: &str, in_hash: &str) -> bool {
    let path = Path(path);

    println!("file_is_fresh: {} {}", path.to_str(), in_hash);

    os::path_exists(&path) && in_hash == digest_file(&path)
}

fn encodable_is_fresh(path: &str, in_hash: &str) -> bool {
    let path = Path(path);

    println!("file_is_fresh: {} {}", path.to_str(), in_hash);

    true
    //in_hash == digest_encodable(&path)
}

impl Context {
    pub fn new(path: Path) -> Context {
        let db = ::workcache::Database::new(path);
        let logger = ::workcache::Logger::new();
        let cfg = TreeMap::new();

        let mut freshness = TreeMap::new();
        freshness.insert(~"path", file_is_fresh);
        freshness.insert(~"encodable", encodable_is_fresh);

        let ctx = ::workcache::Context::new_with(db, logger, cfg, freshness);

        Context { ctx : ctx }
    }

    pub fn prep<'a>(&'a self, fn_name: &'a str) -> ::workcache::Prep<'a> {
        self.ctx.prep(fn_name)
    }
}
