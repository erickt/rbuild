use std::os;
use sync::Future;

use context::Context;

pub fn add_prefix_suffix(mut path: Path, prefix: Option<&str>, suffix: Option<&str>) -> Path {
    match (prefix, suffix) {
        (Some(prefix), Some(suffix)) => {
            let filename = format!("{}{}.{}",
                prefix,
                path.filename_str().unwrap(),
                suffix);

            path.set_filename(filename);
        }
        (Some(prefix), None) => {
            let filename = format!("{}{}",
                prefix,
                path.filename_str().unwrap());

            path.set_filename(filename);
        }
        (None, Some(suffix)) => {
            path.set_extension(suffix);
        }
        (None, None) => { }
    }

    path
}

pub fn find_program(ctx: Context, names: &'static [&'static str]) -> Future<Path> {
    let mut prep = ctx.prep("find_program");
    prep.declare_input("value", "names", &names);

    prep.exec(proc(exec) {
        let paths = os::getenv("PATH").unwrap();

        for name in names.iter() {
            print!("looking for program {}", name);

            let path = Path::new(name.as_slice());
            if path.exists() {
                println!(" ok {}", path.display());
                exec.discover_output_path("output", &path);

                return path;
            }

            for path in paths.split(':') {
                let path = Path::new(path).join(*name);

                if path.exists() {
                    println!(" ok {}", path.display());
                    exec.discover_output_path("output", &path);

                    return path;
                }
            }
        }

        fail!(" failed");
    })
}
