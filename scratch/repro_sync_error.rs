use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = Path::new("test_src");
    let dest = Path::new("test_dest");

    // Setup source
    fs::create_dir_all(src.join("sub"))?;
    fs::write(src.join("file.txt"), "hello")?;
    fs::write(src.join("sub/file2.txt"), "world")?;

    // Setup dest (already exists)
    fs::create_dir_all(dest)?;
    fs::write(dest.join("existing.txt"), "keep me")?;

    println!("Attempting fs_extra merge copy...");
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    options.content_only = true;

    match fs_extra::dir::copy(src, dest, &options) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Failed: {}", e),
    }

    Ok(())
}
