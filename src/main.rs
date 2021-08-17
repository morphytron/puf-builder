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
pub enum EncodingTypes {
    Windows1252, utf8
}
fn main() {
    let mut app = App::new("Atomhid Builder")
        .about("Builds code from templates using csv and template files, and an additional multiplier file.")
        .author("Daniel Alexander Apatiga <daniel.apatiga@eleventh-hour.club>")
        .version("1.0.0")
        .arg(Arg::new("token")
            .short('k')
            .about("Set the token value inside a template txt file.  A parser will look for these tokens, which will have a numeric value corresponding to the column number in the csv file like so: <token><number>.  E.g., ???1")
            .long("token")
            .default_missing_value("???")
            .required(false)
            .takes_value(true))
        .arg(Arg::new("csv_input_fname")
            .short('f')
            .long("csv-input-fname")
            .required(false)
            .takes_value(true))
        .arg(Arg::new("output_fname")
            .short('o')
            .long("output-fname")
            .required(true)
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
        .arg(Arg::new("csv_col_delim")
            .short('l')
            .default_missing_value(",")
            .long("csv_col_delim")
            .required(false)
            .takes_value(true))
        .arg(Arg::new("csv_row_delim")
            .short('w')
            .long("csv-row-delim")
            .required(false)
            .default_missing_value("\n")
            .takes_value(true))
        .arg(Arg::new("big_row_delim")
            .short('b')
            .long("big-row-delim")
            .required(false)
            .default_missing_value(r"pub struct [\w]{1,} \{[\w\s\d\D--\}]+")
            .takes_value(true))
        .arg(Arg::new("append_str")
            .short('a')
            .long("append")
            .required(false)
            .about("Append one file unto another file.")
            .default_missing_value("false"))
        .arg(Arg::new("templ_fname")
            .value_name("templ_fname")
            .short('t')
            .long("template-fname")
            .required(false)
            .takes_value(true))
        .arg(Arg::new("i_encoding")
            .short('d')
            .long("i-encoding")
            .required(false)
            .default_missing_value("utf8")
            .takes_value(true))
        .arg(Arg::new("csvi_encoding")
            .short('z')
            .long("csv-i-encoding")
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
            .default_missing_value("false")
            .takes_value(false))
        .arg(Arg::new("build_by_re")
            .short('y')
            .long("build-by-re")
            .required(false)
            .takes_value(false))
        .arg(Arg::new("is_struct")
            .short('s')
            .long("is-struct")
            .required(false)
            .takes_value(false))
        .arg(Arg::new("col_1_ind")
            .long("col-1-index")
            .required(false)
            .takes_value(true))
        .arg(Arg::new("col_2_ind")
             .long("col-2-index")
             .required(false)
             .takes_value(true))
        .arg(Arg::new("list_funcs")
            .short('u')
            .long("list-functions")
            .required(false)
            .about("This enables the listing of functions generated as a post script building step.")
            .default_missing_value("false")
            .takes_value(false))
        .arg(Arg::new("re_mappings")
            .long("builder-re-mappings")
            .required(false)
            .short('p')
            .takes_value(true));

    let matches = app.get_matches();
    let mut listfuncs = false;
    if matches.occurrences_of("list_funcs") > 0 {
        listfuncs = true;
    }
    let input_file_name = matches.value_of("input_fname").expect("Missing input filename.");
    let csv_input_file_name = matches.value_of("csv_input_fname").unwrap_or_default();
    let tolken = matches.value_of("token").unwrap_or("???");
    let output_file_name = matches.value_of("output_fname").expect("Missing output filename.");
    let delimitter_col = matches.value_of("col_delim").unwrap_or(",");
    let delimitter_row = matches.value_of("row_delim").unwrap_or("\n");
    let csv_delimitter_row = matches.value_of("csv_col_delim").unwrap_or(",");
    let csv_delimitter_col = matches.value_of("csv_row_delim").unwrap_or("\n");
    let delimitter_bigger_row = matches.value_of("big_row_delim").unwrap_or(r"pub struct [\w]{1,} \{[\w\s\d\D--\}]+");
    let mut append = false;
    if matches.occurrences_of("append_str") > 0 {
        append = true;
    }
    let template_file_name = matches.value_of("templ_fname").unwrap_or("");
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
    let mut build_by_re = false;
    if matches.occurrences_of("build_by_re") > 0 {
        build_by_re = true;
    }
    let mut is_struct = false;
    if matches.occurrences_of("is_struct") > 0 {
        is_struct = true;
    }
    let col_1_index = matches.value_of("col_1_ind").unwrap_or("1").parse::<usize>().unwrap();
    let col_2_index = matches.value_of("col_2_ind").unwrap_or("2").parse::<usize>().unwrap();
    let builder_re_mappings = matches.value_of("re_mappings").unwrap_or("-1").parse::<i16>().unwrap();
   //dbg!(input_file_name, output_file_name, template_file_name, append)     ;
    if verbose {
        println!(
            "<Input file: {}, output file: {}, template file: {}>",
            input_file_name, output_file_name, template_file_name
        );
    }
    if append {
        let input_bytes = read_file(input_file_name, i_encoding).unwrap();
        write_file(
            output_file_name,
            &mut String::from_utf8(input_bytes).expect("Found invalid UTF-8"),
            i_encoding
        );
    } else if build_by_re {
        let input_bytes = read_file(input_file_name, i_encoding).unwrap();
        let input_text = String::from_utf8(input_bytes).expect("Found invalid UTF-8");
        let template_bytes = read_file(template_file_name, t_encoding).unwrap();
        let template_text = String::from_utf8(template_bytes).expect("Found invalid UTF-8");
        let input_template_bytes = read_file(csv_input_file_name, csvi_encoding).unwrap();
        let input_template_text =
            String::from_utf8(input_template_bytes).expect("Found invalid UTF-8");
        let mut output = split_input_into_row_set(
            input_template_text.as_str(),
            csv_delimitter_row,
            csv_delimitter_col,
            tolken,
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
        )
            .string()
            .unwrap();
        if output_file_name != "" {
            write_file(output_file_name, &mut output, o_encoding);
        } else {
            println!("{}", output);
        }
    } else if listfuncs {
        let mut output = postBuild(input_file_name, i_encoding);
        if output_file_name != "" {
            write_file(output_file_name, &mut output, o_encoding);
        } else {
            println!("{}", output);
        }
    } else {
        let input_bytes = read_file(input_file_name, i_encoding).unwrap();
        let input_text = String::from_utf8(input_bytes).expect("Found invalid UTF-8");
        let template_bytes = read_file(template_file_name, t_encoding).unwrap();
        let template_text = String::from_utf8(template_bytes).expect("Found invalid UTF-8");
        let mut output = buildOutput(
            &input_text.to_string(),
            template_text.to_string(),
            delimitter_col,
            delimitter_row,
            tolken,
            trim_new_lines,
        );
        if output_file_name != "" {
            write_file(output_file_name, &mut output, o_encoding);
        } else {
            println!("{}", output);
        }
    }
}
