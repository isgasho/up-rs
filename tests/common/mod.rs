/// Common functions that are used by other tests.

extern crate walkdir;

use std::env;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use std::error;
use std::os::unix;

use self::walkdir::WalkDir;

#[cfg(test)]
pub fn dot_cmd() -> Command {
    let dot_path = env::current_exe().unwrap()
        .parent().expect("executable's directory")
        .parent().expect("build directory").join("dot");
    Command::new(&dot_path) }

/// Returns the test module name.
#[cfg(test)]
fn test_module() -> String {
    env::current_exe().unwrap()
        .file_name().unwrap()
        .to_str().unwrap()
        .chars().take_while(|c| *c != '-')
        .collect()
}

/// Returns the path to the tests/fixtures directory (relative to the crate root).
#[cfg(test)]
pub fn fixtures_dir() -> PathBuf {
    env::current_exe().unwrap()
        .parent().expect("executable's directory")
        .parent().expect("build directory")
        .parent().expect("debug/release directory")
        .parent().expect("target directory")
        .join("tests").join("fixtures")
}

/// Returns the path to a temporary directory for your test (OS tempdir + test file name + test function name).
/// Cleans the directory if it already exists.
#[cfg(test)]
pub fn temp_dir(test_fn: &str) -> Result<PathBuf, Box<error::Error>> {
    let mut temp_dir = env::temp_dir();
    temp_dir.push(test_module());
    temp_dir.push(test_fn);
    assert!(temp_dir.starts_with(env::temp_dir()));
    if temp_dir.exists() {
        temp_dir.canonicalize()?;
        fs::remove_dir_all(&temp_dir)?;
    }
    assert!(!temp_dir.exists());
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

#[cfg(test)]
pub fn copy_all(from_dir: &Path, to_dir: &Path) -> Result<(), Box<error::Error>> {
    println!("Copying everything in '{:?}' to '{:?}'", from_dir, to_dir);
    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        {
            let from_path_metadata = from_path.metadata()?;
            let from_path = from_path.path();
            println!("Path: {:?}", &from_path);

            let rel_path = from_path.strip_prefix(&from_dir)?;
            let to_path = to_dir.join(rel_path);

            let file_type = from_path_metadata.file_type();
            fs::create_dir_all(to_path.parent().unwrap())?;
            if file_type.is_dir() {
                fs::create_dir(to_path)?;
            } else if file_type.is_symlink() {
                unix::fs::symlink(fs::read_link(&from_path)?, to_path)?;
            } else if file_type.is_file() {
                fs::copy(from_path, to_path)?;
            }
        }
    Ok(())
}

#[cfg(test)]
/// Panic if there is a file, directory, or link at the path.
pub fn assert_no_file(path: &Path) {
    assert!(! path.exists());
}

#[cfg(test)]
/// Panic if there is not a file at the path, or if the contents don't match.
pub fn assert_file(path: &Path, contents: &str) {
    assert!(path.is_file());
    assert_eq!(fs::read_to_string(path).unwrap(), contents);
}

#[cfg(test)]
/// Panic if there is not a directory at the path.
pub fn assert_dir(path: &Path) {
    assert!(path.is_dir());
}

#[cfg(test)]
/// Panic if there is not a link at the path, or if the destination isn't the one provided.
pub fn assert_link(path: &Path, destination: &Path) {
    assert!(path.exists());
    assert_eq!(fs::read_link(path).unwrap(), destination);
}