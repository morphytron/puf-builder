#![feature(toowned_clone_into)]
extern crate clap;
extern crate regex;
extern crate string_builder;
extern crate encoding_rs;
extern crate encoding_rs_io;
mod builder;
mod io;

use std::env;
use clap::*;
use regex::{Match, Regex};
use crate::io::io::*;
use crate::builder::builder::*;
use std::ops::Deref;

fn main() {
    let mut app = App::new("PUF Builder")
        .about("By using various algorithms, you can create scripts, texts, and other patternable outputs.  This application focuses on simplifying coding where possible.")
        .author("Daniel Alexander Apatiga <daniel.apatiga@pickupfit.com>")
        .version("1.0.0")
        .subcommand(App::new("build")
            .about("This algorithm builds your code from templates using a delimitted csv input file, and a template files.  After transformation, a single output file is concatenated from the algorithm.")
            .arg(Arg::new("token")
                .short('k')
                .about("Set the token value inside a template txt file.  A parser will look for these tokens, which will have a numeric value corresponding to the column number in the csv file like so: <token><number>.  E.g., ???1")
                .long("token")
                .default_missing_value("???")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("col_delim")
                .short('c')
                .long("col_delim")
                .required(false)
                .default_missing_value(",")
                .takes_value(true))
            .arg(Arg::new("row_delim")
                .short('r')
                .long("row_delim")
                .required(false)
                .default_missing_value("\n")
                .takes_value(true))
            .arg(Arg::new("input_fname")
                .short('i')
                .long("input_fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("templ_fname")
                .value_name("templ_fname")
                .short('t')
                .long("template-fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("i_encoding")
                .short('d')
                .long("i-encoding")
                .required(false)
                .default_missing_value("utf8")
                .takes_value(true))
            .arg(Arg::new("o_encoding")
                .short('g')
                .long("o-encoding")
                .default_missing_value("utf8")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("t_encoding")
                .short('g')
                .long("o-encoding")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("verbose")
                .short('v')
                .long("verbose")
                .about("Turn on verbose output.")
                .required(false)
                .default_missing_value("false")
                .takes_value(false))
            .arg(Arg::new("output_fname")
                .short('o')
                .long("output-fname")
                .required(true)
                .takes_value(true))


        )
        .subcommand(App::new("append")
            .about("Append a file's contents into another file, while maintaining encoding restrictions.")
            .arg(Arg::new("output_fname")
                .short('o')
                .long("output-fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("input_fname")
                .short('i')
                .long("input_fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("i_encoding")
                .short('d')
                .long("i-encoding")
                .required(false)
                .default_missing_value("utf8")
                .takes_value(true))
            .arg(Arg::new("t_encoding")
                .short('g')
                .long("o-encoding")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("verbose")
                .short('v')
                .long("verbose")
                .about("Turn on verbose output.")
                .required(false)
                .default_missing_value("false")
                .takes_value(false))

        )
        .subcommand(App::new("buildre")
            .about("This algorithm builds from two input files rather than just one.  Keeping the input csv file, you also specify another file from which you multiply its matrice of rows and columns with the rows and columns of the csv file.")
            .arg(Arg::new("unsafe_row_count")
                .long("disable-row-count-assert")
                .default_missing_value("false")
                .required(false)
                .short('w')
                .takes_value(false))
            .arg(Arg::new("skip_little_rows")
                .long("skip-little-i-rows")
                .about("Skip the little row index for each numeric value in this array, delineated by commas.  (Only works with the input file, not the csv input parameter.)")
                .short('L')
                .default_missing_value("")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("skip_big_rows")
                .long("skip-big-i-rows")
                .about("Skip the big row index for each numeric value in this array, delineated by commas.  (Only works with the input file, not the csv input parameter.)")
                .short('B')
                .default_missing_value("")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("map_row_vs_csv_col_ind")
                .long("map-i-row-vs-col-index")
                .about("If you have a row in a csv file, this value is an index of a column which references the exact big row index of the input file so that it is associated correctly.")
                .short('M')
                .default_missing_value("0")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("re_mappings")
                .long("builder-re-mappings")
                .default_missing_value("-1")
                .required(false)
                .short('p')
                .takes_value(true))
            .arg(
                Arg::new("token")
                .short('k')
                .about("Set the token value inside a template txt file.  A parser will look for these tokens, which will have a numeric value corresponding to the column number in the csv file like so: <token><number>.  E.g., ???1")
                .long("token")
                .default_value("???")
                .default_missing_value("???")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("csv_input_fname")
                .short('i')
                .long("csv-input-fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("output_fname")
                .short('o')
                .long("output-fname")
                .about("Specify the output filename path/filename.ext.  Leave blank to output result into console.")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("col_delim")
                .short('c')
                .long("col_delim")
                .required(false)
                .default_value(",")
                .default_missing_value(",")
                .takes_value(true))
            .arg(Arg::new("row_delim")
                .short('r')
                .long("row_delim")
                .required(false)
                .default_value("\n")
                .default_missing_value("\n")
                .takes_value(true))
            .arg(Arg::new("input_fname")
                .short('f')
                .long("input_fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("csv_col_delim")
                .short('l')
                .default_value(",")
                .default_missing_value(",")
                .long("csv_col_delim")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("csv_row_delim")
                .short('w')
                .long("csv-row-delim")
                .required(false)
                .default_value("\n")
                .default_missing_value("\n")
                .takes_value(true))
            .arg(Arg::new("big_row_delim")
                .short('b')
                .long("big-row-delim")
                .required(false)
                .default_value(r"pub[\s]+struct[\s]+[\w]{1,}[\s]+\{[\w\s\d:;<>,]+")
                .default_missing_value(r"pub[\s]+struct[\s]+[\w]{1,}[\s]+\{[\w\s\d:;<>,]+")
                .takes_value(true))
            .arg(Arg::new("templ_fname")
                .value_name("templ_fname")
                .short('t')
                .long("template-fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("i_encoding")
                .short('d')
                .long("i-encoding")
                .required(false)
                .default_value("utf8")
                .default_missing_value("utf8")
                .takes_value(true))
            .arg(Arg::new("csvi_encoding")
                .short('z')
                .long("csv-i-encoding")
                .required(false)
                .default_value("utf8")
                .default_missing_value("utf8")
                .takes_value(true))
            .arg(Arg::new("o_encoding")
                .short('g')
                .long("o-encoding")
                .default_value("utf8")
                .default_missing_value("utf8")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("t_encoding")
                .short('g')
                .long("o-encoding")
                .required(false)
                .takes_value(true))
            .arg(Arg::new("trim_new_lines")
                .short('m')
                .long("trim-new-lines")
                .required(false)
                .takes_value(false))
            .arg(Arg::new("verbose")
                .short('v')
                .long("verbose")
                .about("Turn on verbose output.")
                .required(false)
                .default_value("false")
                .takes_value(false))
            .arg(Arg::new("col_1_ind")
                .long("col-1-index")
                .required(false)
                .default_value("0")
                .default_missing_value("0")
                .takes_value(true))
            .arg(Arg::new("col_2_ind")
                .long("col-2-index")
                .required(false)
                .default_value("1")
                .default_missing_value("1")
                .takes_value(true))
            .arg(Arg::new("is_struct")
                .short('s')
                .long("is-struct")
                .required(false)
                .takes_value(false))
        ).subcommand(App::new("blist")
            .about("This algorithm builds a list by parsing a file using regex.")
            .arg(Arg::new("is_list_rocket_routes")
                .short('r')
                .long("rust-rocket-routes")
                .required(false)
                .about("This enables the listing of functions generated as a post script building step.")
                .default_missing_value("true")
                .takes_value(false))
            .arg(Arg::new("output_fname")
                .short('o')
                .long("output-fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("input_fname")
                .short('i')
                .long("input_fname")
                .required(true)
                .takes_value(true))
            .arg(Arg::new("verbose")
                .short('v')
                .long("verbose")
                .about("Turn on verbose output.")
                .required(false)
                .default_missing_value("false")
                .takes_value(false))
    );

    let mut app_matches = app.get_matches();
    let subcmd = app_matches.subcommand_name().unwrap_or_default();
    if subcmd != "" {
        let matches = app_matches.subcommand_matches(subcmd).unwrap();
        let mut listfuncs = false;
        if matches.occurrences_of("list_funcs") > 0 {
            listfuncs = true;
        }
        let input_file_name = matches.value_of("input_fname").expect("Missing input filename.");
        let csv_input_file_name = matches.value_of("csv_input_fname").unwrap_or_default();
        let token = matches.value_of("token").unwrap_or("???");
        let output_file_name = matches.value_of("output_fname").unwrap_or("");
        let delimitter_col = matches.value_of("col_delim").unwrap_or(",");
        let delimitter_row = matches.value_of("row_delim").unwrap_or("\n");
        let csv_delimitter_row = matches.value_of("csv_row_delim").unwrap_or("\n");
        let csv_delimitter_col = matches.value_of("csv_col_delim").unwrap_or(",");
        let delimitter_bigger_row = matches.value_of("big_row_delim").unwrap_or(r"pub[\s]+struct[\s]+[\w]{1,}[\s]+\{[\w\s\d:;<>,]+");
        let template_file_name = matches.value_of("templ_fname").unwrap_or("");
        let mut disable_assert_row_count = false;
        if matches.occurrences_of("unsafe_row_count") > 0 {
            disable_assert_row_count = true;
        }
        let mut verbose = false;
        if matches.occurrences_of("verbose") > 0 {
            verbose = true;
        }
        let i_encoding = matches.value_of("i_encoding").unwrap_or("utf8");
        let csvi_encoding = matches.value_of("csvi_encoding").unwrap_or("utf8");
        let o_encoding = matches.value_of("o_encoding").unwrap_or("utf8");
        let t_encoding = matches.value_of("t_encoding").unwrap_or("utf8");
        let mut trim_new_lines = false;
        if matches.occurrences_of("trim_new_lines") > 0 {
            trim_new_lines = true;
        }
        let mut is_struct = false;
        if matches.occurrences_of("is_struct") > 0 {
            is_struct = true;
        }
        let col_1_index = matches.value_of("col_1_ind").unwrap_or("0").parse::<usize>().unwrap();
        let col_2_index = matches.value_of("col_2_ind").unwrap_or("1").parse::<usize>().unwrap();
        //dbg!(input_file_name, output_file_name, template_file_name, append)     ;
        if verbose {
            println!(
                "<Input file: {}, output file: {}, template file: {}>",
                input_file_name, output_file_name, template_file_name
            );
        }

        if subcmd == "append" {
            let input_bytes = read_file(input_file_name, i_encoding).unwrap();
            write_file(
                output_file_name,
                &mut String::from_utf8(input_bytes).expect("Found invalid UTF-8"),
                i_encoding
            );
        } else if subcmd == "buildre" {
            let map_row_vs_csv_col_ind = matches.value_of("map_row_vs_csv_col_ind").unwrap_or("0");
            let map_row_vs_col = map_row_vs_csv_col_ind.parse::<usize>().unwrap();
            let mut l_rows_v : Vec<usize> = Vec::new();
            let skip_l_rows_str = matches.value_of("skip_little_rows").unwrap_or("");
            if skip_l_rows_str != "" {
                let l_rows_to_skip_v : Vec<&str> = skip_l_rows_str.split(",").collect();
                for row in l_rows_to_skip_v {
                    let row_val : usize = row.parse::<usize>().unwrap();
                    l_rows_v.push(row_val);
                }
            }
            let mut rows_v : Vec<usize> = Vec::new();
            let skip_b_row_str = matches.value_of("skip_big_rows").unwrap_or("");
            if skip_b_row_str != "" {
                let rows_to_skip_v : Vec<&str> = skip_b_row_str.split(",").collect();
                for row in rows_to_skip_v {
                    let row_val : usize = row.parse::<usize>().unwrap();
                    rows_v.push(row_val);
                }
            }
            let builder_re_mappings = matches.value_of("re_mappings").unwrap_or("-1").parse::<i16>().unwrap();
            let input_bytes = read_file(input_file_name, i_encoding).unwrap();
            let input_text = String::from_utf8(input_bytes).expect("Found invalid UTF-8");
            let template_bytes = read_file(template_file_name, t_encoding).unwrap();
            let template_text = String::from_utf8(template_bytes).expect("Found invalid UTF-8");
            let input_template_bytes = read_file(csv_input_file_name, csvi_encoding).unwrap();
            let input_template_text =
                String::from_utf8(input_template_bytes).expect("Found invalid UTF-8");
            if verbose {
                dbg!(input_template_text.as_str(), csv_delimitter_row, csv_delimitter_col,token, trim_new_lines, input_text.as_str(), col_1_index, col_2_index, delimitter_bigger_row, delimitter_row, delimitter_col, is_struct, template_text.as_str(), verbose, builder_re_mappings, disable_assert_row_count);
            }
            let mut output = start_buildre(
                input_template_text.as_str(),
                csv_delimitter_row,
                csv_delimitter_col,
                token,
                trim_new_lines,
                input_text.as_str(),
                col_1_index,
                col_2_index,
                delimitter_bigger_row,
                delimitter_row,
                delimitter_col,
                is_struct,
                template_text.as_str(),
                verbose,
                builder_re_mappings,
                disable_assert_row_count,
                rows_v,
                l_rows_v,
                map_row_vs_col
            )
                .string()
                .unwrap();
            //dbg!(&output);
            if output_file_name != "" {
                write_file(output_file_name, &mut output, o_encoding);
            } else {
                println!("{}", output);
            }
        } else if subcmd == "blist" {
            let mut output = postBuild(input_file_name, i_encoding);
            if output_file_name != "" {
                write_file(output_file_name, &mut output, o_encoding);
            } else {
                println!("{}", output);
            }
        } else if subcmd == "build" {
            let input_bytes = read_file(input_file_name, i_encoding).unwrap();
            let input_text = String::from_utf8(input_bytes).expect("Found invalid UTF-8");
            let template_bytes = read_file(template_file_name, t_encoding).unwrap();
            let template_text = String::from_utf8(template_bytes).expect("Found invalid UTF-8");
            let mut output = buildOutput(
                &input_text.to_string(),
                template_text.to_string(),
                delimitter_col,
                delimitter_row,
                token,
                trim_new_lines,
            );
            if output_file_name != "" {
                write_file(output_file_name, &mut output, o_encoding);
            } else {
                println!("{}", output);
            }
        }
    }
}
