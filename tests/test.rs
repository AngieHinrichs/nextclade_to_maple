use nextclade_to_maple::{Config, nextclade_to_maple};
mod common;
use common::compare_files;

use std::fs;

#[test]
fn basic() {
    let config = Config {
        nextclade_file: "tests/input/basic.nextclade.tsv".to_string(),
        maple_file: "tests/output/basic.mpl".to_string(),
        ref_len: 29903,
        ..Default::default()
    };
    fs::create_dir_all("tests/output").expect("Error creating output directory");
    nextclade_to_maple(config).expect("Error running nextclade_to_maple");
    compare_files("tests/output/basic.mpl", "tests/expected/basic.mpl").expect("Output file does not match expected file");
}

#[test]
fn mask_rename() {
    let config = Config {
        nextclade_file: "tests/input/basic.nextclade.tsv".to_string(),
        maple_file: "tests/output/maskRename.mpl".to_string(),
        ref_len: 29903,
        mask_bed_file: "tests/input/mask.bed".to_string(),
        rename_or_prune_file: "tests/input/rename.tsv".to_string(),
        ..Default::default()
    };
    fs::create_dir_all("tests/output").expect("Error creating output directory");
    nextclade_to_maple(config).expect("Error running nextclade_to_maple");
    compare_files("tests/output/maskRename.mpl", "tests/expected/maskRename.mpl").expect("Output file does not match expected file");
}

#[test]
fn max_substitutions() {
    let config = Config {
        nextclade_file: "tests/input/basic.nextclade.tsv".to_string(),
        maple_file: "tests/output/maxSubst.mpl".to_string(),
        ref_len: 29903,
        max_substitutions: 50,
        ..Default::default()
    };
    fs::create_dir_all("tests/output").expect("Error creating output directory");
    nextclade_to_maple(config).expect("Error running nextclade_to_maple");
    compare_files("tests/output/maxSubst.mpl", "tests/expected/maxSubst.mpl").expect("Output file does not match expected file");
}

#[test]
fn min_real() {
    let config = Config {
        nextclade_file: "tests/input/basic.nextclade.tsv".to_string(),
        maple_file: "tests/output/minReal.mpl".to_string(),
        min_real: 29800,
        ..Default::default()
    };
    fs::create_dir_all("tests/output").expect("Error creating output directory");
    nextclade_to_maple(config).expect("Error running nextclade_to_maple");
    compare_files("tests/output/minReal.mpl", "tests/expected/minReal.mpl").expect("Output file does not match expected file");
}
