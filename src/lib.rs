use std::error::Error;
use std::collections::HashMap;
use std::{io, fs};

use unwrap::unwrap;
use csv::ReaderBuilder;

use std::ops::Bound::{Included, Excluded};
use unbounded_interval_tree::interval_tree::IntervalTree;

#[derive(Default)]
pub struct Config {
    pub nextclade_file: String,
    pub maple_file: String,
    pub mask_bed_file: String,
    pub max_substitutions: usize,
    pub min_real: usize,
    pub ref_len: usize,
    pub rename_or_prune_file: String,
    pub version: bool,
}

fn maybe_interval_tree_from_bed_file(path: &str) -> Option<IntervalTree<usize>> {
    if path.is_empty() {
        return None;
    }
    let mut rdr = ReaderBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(false)
                    .from_path(path)
                    .unwrap();
    let mut tree = IntervalTree::default();
    let mut first_chrom: Option<String> = None;
    for result in rdr.deserialize() {
        let (chrom, start, end):(String, usize, usize) = result.unwrap();
        if first_chrom.is_none() {
            first_chrom = Some(chrom);
        } else if Some(&chrom) != first_chrom.as_ref() {
            panic!("MAPLE can have only one reference, but found {} and {} in --mask_bed_file {path}", first_chrom.unwrap(), &chrom);
        }
        tree.insert((Included(start), Excluded(end)));
    }
    Some(tree)
}

fn maybe_hash_from_two_column_file(path: &str) -> Option<HashMap<String, String>> {
    if path.is_empty() {
        return None;
    }
    let mut map = HashMap::new();
    let mut rdr = ReaderBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(false)
                    .from_path(path)
                    .unwrap();
    for result in rdr.deserialize() {
        let (k, v) = result.unwrap();
        map.insert(k, v);
    }
    Some(map)
}

fn maybe_lookup_name<'a>(rename_hash:&'a Option<HashMap<String, String>>, name_in: &'a str) -> Option<&'a str> {
    // Rename if rename_or_prune_file is provided, prune if not found
    let name_out = if let Some(rename_hash) = rename_hash {
        let name_found = rename_hash.get(name_in);
        if name_found.is_none() {
            None
        } else {
            let name_found = name_found.unwrap();
            Some(name_found.as_ref())
        }
    } else {
        Some(name_in)
    };
    name_out
}

fn get_column<'a>(row:&'a HashMap<String, String>, column:&str) -> &'a str {
    let col = row.get(column);
    let col = unwrap!(col, "Required column {column} not found");
    col
}

fn get_column_usize(row: &HashMap<String, String>, column: &str) -> usize {
    let val: Result<usize, _> = get_column(row, column).parse();
    let val = unwrap!(val, "Error parsing {column}: {}", get_column(row, column));
    val
}

struct MapleDiff {
    t_start: usize,
    length: usize,
    q_base: char,
}

fn print_one_diff(stream_out: &mut Box<dyn io::Write>, t_start: usize, q_base: char, length: usize) {
    // Print one MAPLE diff vs. reference.  t_start is 0-based, 1-based in output.
    if q_base == '-' || q_base == 'n' {
        writeln!(stream_out, "{}\t{}\t{}", q_base, t_start + 1, length).ok().or_else(|| panic!("Error writing to stream"));
    } else {
        writeln!(stream_out, "{}\t{}", q_base, t_start + 1).ok().or_else(|| panic!("Error writing to stream"));
    }
}

fn print_one_masked(stream_out: &mut Box<dyn io::Write>, t_start: usize, q_base: char, length: usize, mask_tree: &Option<IntervalTree<usize>>) {
    // Print one MAPLE diff vs. reference, but only the parts that do not overlap with mask_tree
    if let Some(mask_tree) = mask_tree {
        let diff_range = t_start..t_start+length;
        let overlaps = mask_tree.get_interval_overlaps(&diff_range);
        if overlaps.is_empty() {
            print_one_diff(stream_out, t_start, q_base, length);
        } else {
            let mut diff_start = t_start;
            let mut diff_len = length;
            for (mask_start_bound, mask_end_bound) in overlaps {
                match (mask_start_bound, mask_end_bound) {
                    (Included(mask_start), Excluded(mask_end)) => {
                        if diff_start < *mask_start {
                            print_one_diff(stream_out, diff_start, q_base, *mask_start - diff_start);
                        }
                        let mut len = *mask_end - diff_start;
                        if len > diff_len {
                            len = diff_len;
                        }
                        diff_start = *mask_end;
                        diff_len -= len;
                    }
                    _ => {
                        panic!("Unexpected mask range {:?}", (mask_start_bound, mask_end_bound));
                    }
                }
            }
           
            if diff_len > 0 {
                print_one_diff(stream_out, diff_start, q_base, diff_len);
            }
        }
    } else {
        print_one_diff(stream_out, t_start, q_base, length);
    }
}

fn parse_base(base: Option<char>, description: &str) -> char {
    let base = unwrap!(base, "Expected {description}, but got '{:?}'", base);
    if base != 'A' && base != 'T' && base != 'C' && base != 'G' && base != 'a' && base != 't' && base != 'c' && base != 'g' {
        panic!("Expected [ACGT] base {description}, but got '{:?}'", base);
    }
    let base = base.to_lowercase().next();
    let base = unwrap!(base, "Error lowercasing already-parsed base '{:?}'", base);
    base
}

fn parse_usize(s: &str, description: &str) -> usize {
    let val:Result<usize, _> = s.parse();
    let val = unwrap!(val, "Expected {description}, but got '{:?}'", s);
    val
}

fn substitutions_to_maple(subs: &str) -> Vec<MapleDiff> {
    // Parse values from nextclade TSV substitution column, e.g. "C241T,T670G,C897A,G2060A,C2790T"
    let mut maple_diffs = Vec::new();
    for sub in subs.split(",") {
        if sub.is_empty() {
            continue;
        }
        let _ref_base = parse_base(sub.chars().next(), "start of substitution");
        let pos = parse_usize(&sub[1..sub.len()-1], "position between ref and alt bases in substitution");
        let q_base = parse_base(sub.chars().last(), "end of substitution");
        maple_diffs.push(MapleDiff {
            t_start: pos - 1,
            length: 1,
            q_base,
        });
    }
    maple_diffs
}

fn deletions_to_maple(deletions: &str) -> Vec<MapleDiff> {
    // Parse values from nextclade TSV deletion column, e.g. "123-125,234-236,345-347"
    let mut maple_diffs = Vec::new();
    for deletion in deletions.split(",") {
        if deletion.is_empty() {
            continue;
        }
        if let Some(dash_offset) = deletion.find('-') {
            let start = parse_usize(&deletion[..dash_offset], "start of deletion");
            let end = parse_usize(&deletion[dash_offset+1..], "end of deletion");
            maple_diffs.push(MapleDiff {
                t_start: start - 1,
                length: end - start + 1,
                q_base: '-',
            });
        } else {
            let start = parse_usize(deletion, "single position in deletion");
            maple_diffs.push(MapleDiff {
                t_start: start - 1,
                length: 1,
                q_base: '-',
            });
        }
    }
    maple_diffs
}

fn missing_to_maple(missing: &str) -> Vec<MapleDiff> {
    // Parse values from nextclade TSV missing column, e.g. "8-216,7201,26975-27040"
    let mut maple_diffs = Vec::new();
    for missing in missing.split(",") {
        if missing.is_empty() {
            continue;
        }
        if let Some(dash_offset) = missing.find('-') {
            let start = parse_usize(&missing[..dash_offset], "start of missing");
            let end = parse_usize(&missing[dash_offset+1..], "end of missing");
            maple_diffs.push(MapleDiff {
                t_start: start - 1,
                length: end - start + 1,
                q_base: 'n',
            });
        } else {
            let start = parse_usize(missing, "single position in missing");
            maple_diffs.push(MapleDiff {
                t_start: start - 1,
                length: 1,
                q_base: 'n',
            });
        }
    }
    maple_diffs
}

fn non_acgtns_to_maple(non_acgtns: &str) -> Vec<MapleDiff> {
    // Parse values from nextclade TSV non_acgtns column, e.g. "Y:4321,Y:12846-12847,R:20055"
    let mut maple_diffs = Vec::new();
    for non_acgtn in non_acgtns.split(",") {
        if non_acgtn.is_empty() {
            continue;
        }
        let (q_base, positions) = unwrap!(non_acgtn.split_once(":"), "Expected nonACGTNs to contain a base and a numeric position after ':', but got '{:?}'", non_acgtn);
        if let Some(dash_offset) = positions.find('-') {
            let start = parse_usize(&positions[..dash_offset], "start of range in nonACGTNs");
            let end = parse_usize(&positions[dash_offset+1..], "end of range in nonACGTNs");
            let q_base = q_base.to_lowercase().chars().next().expect("Error lowercasing base");
            maple_diffs.push(MapleDiff {
                t_start: start - 1,
                length: end - start + 1,
                q_base,
            });
        } else {
            let start = parse_usize(positions, "single position in nonACGTNs");
            let q_base = q_base.to_lowercase().chars().next().expect("Error lowercasing base");
            maple_diffs.push(MapleDiff {
                t_start: start - 1,
                length: 1,
                q_base,
            });
        }
    }
    maple_diffs
}

fn md_total_bases(missing: &[MapleDiff]) -> usize {
    missing.iter().map(|diff| diff.length).sum()
}

fn nextclade_to_maple_one_row(row: &HashMap<String, String>, rename_hash: &Option<HashMap<String, String>>,
                              mask_tree: &Option<IntervalTree<usize>>, min_real: usize, max_substitutions: usize,
                              ref_len: usize, stream_out: &mut Box<dyn io::Write>) -> Result<(), Box<dyn Error>> {
    // If alignmentStart is the empty string, then nextclade was not able to align this item; skip it
    if get_column(row, "alignmentStart").is_empty() {
        return Ok(());
    }
    // Parse values from row, see if item should be skipped
    let name = maybe_lookup_name(rename_hash, get_column(row, "seqName"));
    if name.is_none() {
        return Ok(());
    }
    let name = name.unwrap();
    let alignment_start = get_column_usize(row, "alignmentStart");
    let alignment_end = get_column_usize(row, "alignmentEnd");
    let substitutions = substitutions_to_maple(get_column(row, "substitutions"));
    if max_substitutions > 0 && substitutions.len() > max_substitutions {
        return Ok(());
    }
    let deletions = deletions_to_maple(get_column(row, "deletions"));
    let missing = missing_to_maple(get_column(row, "missing"));
    let num_real = alignment_end - alignment_start + 1 - md_total_bases(&missing);
    if num_real < min_real {
        return Ok(());
    }
    let non_acgtns = non_acgtns_to_maple(get_column(row, "nonACGTNs"));

    // Concatenate and sort all diffs
    let mut all_diffs = Vec::new();
    all_diffs.extend(substitutions);
    all_diffs.extend(deletions);
    all_diffs.extend(missing);
    all_diffs.extend(non_acgtns);
    all_diffs.sort_by_key(|diff| diff.t_start);

    // Print MAPLE header
    writeln!(stream_out, ">{}", name).ok().or_else(|| panic!("Error writing to stream"));
    // If alignment_start is not 1, then print Ns from the beginning of reference to the first diff
    if alignment_start > 1 {
        print_one_masked(stream_out, 0, 'n', alignment_start - 1, mask_tree);
    }
    // Print all sorted diffs
    for diff in all_diffs {
        print_one_masked(stream_out, diff.t_start, diff.q_base, diff.length, mask_tree);
    }
    // If we know ref_len and alignment_end is not the last position in the reference, then print Ns after alignment_end.
    if ref_len > 0 && alignment_end < ref_len {
        print_one_masked(stream_out, alignment_end, 'n', ref_len - alignment_end, mask_tree);
    }
    Ok(())
}

pub fn nextclade_to_maple(config: Config) -> Result<(), Box<dyn Error>> {
    if config.version {
        println!("nextclade_to_maple v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    // Open the nextclade TSV file, or stdin if no file is provided
    let stream_in:Box<dyn io::Read> = if !config.nextclade_file.is_empty() {
        Box::new(fs::File::open(config.nextclade_file)?)
    } else {
        Box::new(io::stdin())
    };
    let mut rdr = ReaderBuilder::new()
                    .delimiter(b'\t')
                    .from_reader(stream_in);

    let mut stream_out:Box<dyn io::Write> = if !config.maple_file.is_empty() {
        Box::new(io::BufWriter::new(fs::File::create(config.maple_file)?))
    } else {
        Box::new(io::BufWriter::new(io::stdout()))
    };

    let rename_hash = maybe_hash_from_two_column_file(&config.rename_or_prune_file);
    let mask_tree = maybe_interval_tree_from_bed_file(&config.mask_bed_file);

    for result in rdr.deserialize() {
        let row: HashMap<String, String> = result?;
        nextclade_to_maple_one_row(&row, &rename_hash, &mask_tree, config.min_real, config.max_substitutions, config.ref_len, &mut stream_out)?;
    }
    Ok(())
}
