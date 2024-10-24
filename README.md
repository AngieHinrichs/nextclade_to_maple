# nextclade_to_maple
-----

## Installation

For now, go to https://github.com/AngieHinrichs/nextclade_to_maple/releases/ and click to download the package for your OS and architecture.
For example, if you are on a Linux machine and `uname -m` says `x86_64`, then download nextclade_to_maple-x86_64-unknown-linux-gnu.
That will download the file nextclade_to_maple-x86_64-unknown-linux-gnu.zip to your machine; unzip that to get the nextclade_to_maple executable.

If you would prefer to install through conda, docker or some other means, then please [file an issue](https://github.com/AngieHinrichs/nextclade_to_maple/issues/new/choose) to let me know!

## Usage

By default nextclade_to_maple operates as a pipe from stdin to stdout, converting the TSV output of [nextclade](https:://clades.nextstrain.org/) to the input format expected by [MAPLE](https://github.com/NicolaDM/MAPLE) which is also an input format for [UShER](https://github.com/yatisht/UShER).

Run `nextclade_to_maple --help` to see descriptions of command line options:

```console
Usage:
  nextclade_to_maple [OPTIONS]

Convert Nextclade TSV to MAPLE format

Optional arguments:
  -h,--help             Show this help message and exit
  -i,--input INPUT      Path to the Nextclade TSV file (default: stdin)
  -o,--output OUTPUT    Path to the MAPLE file (default: stdout)
  -r,--ref_len REF_LEN  Length of the reference sequence (default: unknown Ns
                        after end of alignment)
  --max_substitutions MAX_SUBSTITUTIONS
                        Maximum number of substitutions to retain item in
                        output
  --min_real MIN_REAL   Minimum number of real (non-N aligned) bases to retain
                        item in output
  --rename_or_prune RENAME_OR_PRUNE
                        Path to a two-column tab-separated file mapping old
                        names to new names. Drop items with old names not in
                        the file.
  --mask_bed MASK_BED   Path to a BED file (3-6 columns) with regions to mask,
                        i.e. positions will be excluded from the output.
```

## License

`nextclade_to_maple` is distributed under the terms of the [MIT](https://opensource.org/license/mit) license.

