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

// static strings
static HELP: &'static str = r##"
*****General usage of this program*****
For configuring TLS, port, and server address, configure the Rocket.toml or environment variables (in the same folder as this application) as described on their website:
https://www.rocket.rs/

-postgresconns -- sets the number of asynchronous postgresql connections to make.
-v -- indicate verbose output.
-append -- This indicates it will append file contents from an input file to an output file. As opposed to using Shell, Bash, or Powerpoint/CMD, it maintains utf-8 encoding. It also prevents the application from running as a server.

For template building only (basic usage):
pufm -i <input-file-path (of CSV-formatted file)> -t <template file path> -o <output file path>

For advanced usage, these additional parameters are availble:
-i-csv-dr -- the regexp for delimitting the row of the CSV input file. Default is newline aka. "\n".
-i-csv-dc -- the regexp for delimitting the column of each row.  Default is comma aka. ",".
-tolken <a unique tolken string> -- this indicates the unique tolken value proceeded by the index of the column (starting with value 1) for parsing the template (where to replace the values from the input to the template.)  The default is ???.
-builder -- this prevents the application from running as an API service for Pickup Fitness, indicating that it is to be used for building scripts or files from a template, input (to an output.)  Aka., this arg prevents PostgreSQL connections from establishing and binding the IP address and port for the RESTful Web Service.
-listfuncs -- this is strictly for listing the functions generated as a post script building step.
-builder-re -- this requires additional parameters to be added: -i-csv <file path>, and more described below.  This will make additional string tolkens available within the template file given the parsed input:
  %COLS% : If the program is run with -is-struct argument, this tolken will list each public variable in a struct of a file as a list, e.g. "username, password"
  %COLS_PLACEMENTS% :  the usage of this tolken is for SQL-templated coding.  If the program is run with -is-struct argument, this tolken will list format-macro-compatible placement tolkens for Rust handling of formatting the stringed SQL statement.  For example, it could look like this {} '{}' {} depending on whether the variable is a string or not.
  %OBJ_COLS% : If the program is run with -is-struct argument, this tolken will generate a list of Rust functions that will extract the data from serializable Json struct.  It takes into consideration whether the struct variable is an Option enum and the output looks like this: obj.variable_name, obj.variable_name.unwrap()
-is-struct -- this works only with -builder-re... it indicates that the input-argument file will be parsed for Rust structs.

If not using -is-struct while using -builder-re mode, add additional args for parsing the input file.  These are:
-i-dR -- this regexp will split the file into groups for parsing the rows and columns.
-i-dr -- this regexp will retrieve the rows from each respective grouping.
-i-dc -- this regexp will split the row as delimitted.

-builder-re-column-for-map -- this works only in conjunction with -builder-re.  Semantically, for the correct mapping of the csv row with its corresponding input-parsed row, a column index is necessary as a pointer to the string/regexpression for correctly locating it.
    An example: -builder-re-mapping 5.
"##;
fn main() {
    let mut app = App::new("Atomhid Coder")
        .about("By using various algorithms, you can create scripts, texts, and other patternable outputs.  This application focuses on simplifying coding where possible.")
        .author("Daniel Alexander Apatiga <daniel.apatiga@eleventh-hour.club>")
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
            .arg(Arg::new("skip_cols")
                .long("skip-cols")
                .about("Skip the column index for each numeric value in this array, delineated by commas.  (Only works with the input file, not the csv input parameter.)")
                .short('S')
                .required(false)
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
                .default_value(r"pub struct [\w]{1,} \{[\w\s\d\D--\}]+")
                .default_missing_value(r"pub struct [\w]{1,} \{[\w\s\d\D--\}]+")
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
        let delimitter_bigger_row = matches.value_of("big_row_delim").unwrap_or(r"pub struct [\w]{1,} \{[\w\s\d\D--\}]+");
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
            let skip_cols_str = matches.value_of("skip_cols").unwrap_or("");
            let cols_to_skip_v : Vec<&str> = skip_cols_str.split(",").collect();
            let mut cols_v : Vec<usize> = Vec::new();
            for col in cols_to_skip_v {
                let col_val : usize = col.parse::<usize>().unwrap();
                cols_v.push(col_val);
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
                cols_v
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
