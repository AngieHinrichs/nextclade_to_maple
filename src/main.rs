use std::process;

use argparse::{ArgumentParser, Store};

use nextclade_to_maple::{Config, nextclade_to_maple};

fn parse_args() -> Config {
    let mut nextclade_file = String::new();
    let mut maple_file = String::new();
    let mut mask_bed_file = String::new();
    let mut max_substitutions = 0;
    let mut min_real = 0;
    let mut ref_len = 0;
    let mut rename_or_prune_file = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Convert Nextclade TSV to MAPLE format");
        ap.refer(&mut nextclade_file)
            .add_option(&["-i", "--input"], Store,
                        "Path to the Nextclade TSV file (default: stdin)");
        ap.refer(&mut maple_file)
            .add_option(&["-o", "--output"], Store,
                        "Path to the MAPLE file (default: stdout)");
        ap.refer(&mut ref_len)
            .add_option(&["-r", "--ref_len"], Store,
                        "Length of the reference sequence (default: unknown Ns after end of alignment)");
        ap.refer(&mut max_substitutions)
            .add_option(&["--max_substitutions"], Store,
                        "Maximum number of substitutions to retain item in output");
        ap.refer(&mut min_real)
            .add_option(&["--min_real"], Store,
                        "Minimum number of real (non-N aligned) bases to retain item in output");
        ap.refer(&mut rename_or_prune_file)
            .add_option(&["--rename_or_prune"], Store,
                        "Path to a two-column tab-separated file mapping old names to new names. Drop items with old names not in the file.");
        ap.refer(&mut mask_bed_file)
            .add_option(&["--mask_bed"], Store,
                        "Path to a BED file (3-6 columns) with regions to mask, i.e. positions will be excluded from the output.");
        ap.parse_args_or_exit();
    }

    Config {
        nextclade_file,
        maple_file,
        mask_bed_file,
        max_substitutions,
        min_real,
        ref_len,
        rename_or_prune_file,
    }
}

fn main() {
    let config = parse_args();
    if let Err(err) = nextclade_to_maple(config) {
        println!("{}", err);
        process::exit(1);
    }
}

