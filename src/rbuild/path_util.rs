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
