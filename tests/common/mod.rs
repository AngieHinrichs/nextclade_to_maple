use std::fs;
use std::error::Error;

pub fn compare_files(path1: &str, path2: &str) -> Result<(), Box<dyn Error>> {
    let content1 = fs::read_to_string(path1)?;
    let content2 = fs::read_to_string(path2)?;
    let content2 = content2.replace("\r", "");
    assert_eq!(content1, content2);
    Ok(())
}
