pub mod builder {
    use crate::io::io::{read_file, write_file};
    use crate::lazy_static::lazy_static;
    use regex::{Match, Regex};
    use std::borrow::{Borrow, BorrowMut};
    use std::cell::{RefCell, RefMut};
    use std::convert::TryFrom;
    use std::ops::{Deref, DerefMut};
    use std::sync::{Mutex, MutexGuard};
    use string_builder::Builder;
    lazy_static! {
        static ref return_on_col_pattern: Regex = Regex::new("[\r\n]*").unwrap();
    }
    fn omit_rows_by_re<'a>(omit_csv_row_re :&str, mut cols : Vec<&'a str>) -> Vec<&'a str> {
        let csv_row_re = Regex::new(omit_csv_row_re);
        if let Ok(re) = csv_row_re {
            cols = cols.iter().filter(|r| {
                let c = re.captures(r);
                if let Some(cap) = c {
                    if cap.len() > 0 {
                        return false;
                    }
                    return true;
                }
                true
            }).map(|r| *r).collect();
        } else {
            panic!("Could not create Regex from '{}'.", omit_csv_row_re);
        }
        cols
    }

    fn omit_cols_by_re(
        omit_csv_col_re: &str,
        temp: String,
        mut cols: Vec<String>,
        token: &str,
        trim_endlines: bool,
        verbose: bool,
    ) -> (String, Vec<String>) {
        let re = Regex::new(omit_csv_col_re);
        if verbose {
            println!("Omit csv_col_re value has been set to regular expression: {}", omit_csv_col_re);
        }
        let mut final_string = String::new();
        if let Ok(rege) = re.clone() {
            let mut index = 0;
            for col in cols.iter_mut() {
                if let Some(captur) = rege.captures(col.as_str()) {
                    if captur.len() > 0 {
                        if verbose {
                            println!("Omitting: <regex: {}, column: {}>", col, rege.as_str());
                        }
                        if trim_endlines {
                            return_on_col_pattern.replace(col, "").to_string();
                        }
                        *col = "".to_string();
                        //dbg!(col);
                        final_string = replace_token_in_template(
                            token,
                            index.clone(),
                            temp.to_string(),
                            col,
                            verbose,
                        );
                    }
                } else {
                    if verbose {
                        println!("Not omitting: <column: {}>", col.as_str());
                    }
                    if trim_endlines {
                        return_on_col_pattern.replace(col, "").to_string();
                    }
                    //dbg!(col);
                    final_string = replace_token_in_template(token, index, temp.to_string(), col, verbose);
                    index += 1;
                }
            }
        } else {
            panic!(
                "Could not create a reg. expression from  --omit-csv-col-re value: '{}'.",
                omit_csv_col_re
            );
        }
        (final_string, cols)
    }
    fn replace_token_in_template(
        token: &str,
        index: usize,
        template: String,
        col: &str,
        verbose: bool,
    ) -> String {
        let token_replacement = format!("{0}{1}", token.to_string(), index.to_string());
        let token_matcher_re = token_replacement.clone() + "<<regex:.*>>";
        let token_matcher_reg = Regex::new(token_matcher_re.as_str());
        //get each match for the token.
        let mut templ_clone = template.clone();
        if let Ok(token_matcher_rege) = token_matcher_reg {
            let temp_template = templ_clone.clone();
            for mat in token_matcher_rege.find_iter(temp_template.as_str()) {
                if verbose {
                    println!(
                        "Found match for regex parameter by token:  <regex:{}>.",
                        mat.as_str()
                    );
                }
                let start = mat.start();
                let stop = mat.end();
                let first_part_template = &template[0..start];
                let token_str = mat.as_str();
                let rege_isolator =
                    "(?:".to_string() + token_replacement.as_str() + "<<regex:)(?P<re>.*)(?:>>)";
                let regex_isolator_re = Regex::new(rege_isolator.as_str()).unwrap();
                let captures = regex_isolator_re.captures(token_str);
                if let Some(cap) = captures {
                    let regex_specified = &cap["re"];
                    if verbose {
                        println!("Regex specified: {}.", regex_specified);
                    }
                    let re_specified_ = Regex::new(regex_specified);
                    if let Ok(re_specified) = re_specified_ {
                        let col_captures = re_specified.captures(col);
                        if let Some(cap) = col_captures {
                            let match_ = &cap["a"];
                            templ_clone = first_part_template.to_string() + match_+ &templ_clone[stop..template.len()];
                            if verbose {
                                println!("The column slice is '{}'", match_);
                                println!("The new template looks like: '{}'.", templ_clone.as_str());
                            }
                        } else {
                            templ_clone = templ_clone.replace(mat.as_str(), col);
                            if verbose {
                                println!("Did did not capture from the regex, '{}'... so, replaced '{}' with '{}'.", regex_specified, mat.as_str(), col);
                            }
                        }
                    }
                    else {
                        panic!("Could not compile regex: '{}'", regex_specified);
                    }
                } else {
                    panic!("Could not find regexp in the <<regex::exp> expression located at char position {}.", start);
                }
            }
        } else if verbose {
            println!("Token did not have a regex qualifier...");
        }
        templ_clone = templ_clone.replace(token_replacement.as_str(), col);
        templ_clone
    }
    // Called by buildre function
    pub fn buildOutputFromEntry(
        entry: String,
        template: &mut String,
        col_del: &str,
        token: &str,
        trim_endlines: bool,
        verbose: bool,
        omit_csv_col_re: &str,
    ) -> String {
        println!("In buildOutputFromEntry function.");
        //dbg!(entry, &template, col_del, token, trim_endlines);
        let mut cols: Vec<String> = Vec::new();
        for c in entry.split(col_del) {
            cols.push(c.to_string());
        }
        let mut index = 1;
        if omit_csv_col_re != "" {
            let (t, cols_) = omit_cols_by_re(
                omit_csv_col_re,
                template.clone(),
                cols,
                token,
                trim_endlines,
                verbose,
            );
            *template = t;
            cols = cols_;
        } else {
            //dbg!(&cols);
            for col in &cols {
                if trim_endlines {
                    return_on_col_pattern.replace(col, "").to_string();
                }
                //dbg!(col);
                *template =
                    replace_token_in_template(token, index, template.to_string(), col, verbose);
                //dbg!(&template);
                //println!("{}",clone);
                index += 1;
            }
        }
        template.clone()
    }

    // Called by build function
    pub fn buildOutput(
        input: &String,
        mut template: String,
        col_del: &str,
        row_del: &str,
        token: &str,
        trim_endlines: bool,
        verbose: bool,
        omit_csv_csv_re: &str,
        omit_csv_row_re: &str,
    ) -> String {
        let mut builder = Builder::default();
        let mut rows: Vec<&str> = input.as_str().split(row_del).collect();
        if omit_csv_row_re != "" {
            if verbose {
                println!("Omitting rows by re...");
            }
            rows = omit_rows_by_re(omit_csv_row_re, rows);
            if verbose{
                println!("Rows length is {}.", rows.len());
            }
        }
        for row in rows {
            let mut clone = template.clone();
            let mut cols: Vec<String> = Vec::new();
            for c in row.split(col_del) {
                cols.push(c.to_string());
            }
            // let size = cols.len();
            if omit_csv_csv_re != "" {
                let(c_, cols_) = omit_cols_by_re(
                    omit_csv_csv_re,
                    clone.clone(),
                    cols,
                    token,
                    trim_endlines,
                    verbose,
                );
                clone = c_;
                cols = cols_;
            }
            let mut index = 1;
            for col in cols.iter() {
                let mut token_replacement = token.to_string();
                if trim_endlines {
                    let col_size = token_replacement.len();
                    token_replacement = token_replacement
                        .as_str()
                        .trim_end_matches("\r")
                        .to_string();
                    token_replacement = token_replacement.trim_end_matches("\n").to_string();
                }
                clone = replace_token_in_template(
                    token_replacement.as_str(),
                    index,
                    clone.clone(),
                    col,
                    verbose,
                );
                //println!("{}",clone);
                index += 1;
            }
            builder.append(clone.as_str());
        }
        builder.string().unwrap()
    }

    pub fn split_csv_row_into_vec<'a>(csv_row: &'a str, col_del: &'a str) -> Vec<&'a str> {
        let cols: Vec<&'a str> = csv_row.split(col_del).collect::<Vec<&'a str>>();
        cols
    }

    pub fn start_buildre(
        input: &str,
        csv_row_del: &str,
        csv_col_del: &str,
        token: &str,
        trim_endlines: bool,
        regexable_input: &str,
        ind_1: usize,
        ind_2: usize,
        row_regex_str: &str,
        split_r_by: &str,
        split_column_by: &str,
        is_struct: bool,
        mut template: String,
        is_verbose: bool,
        builder_re_mapping: i16,
        disable_assert_row_count: bool,
        big_rows_to_skip: Vec<usize>,
        little_rows_to_skip: Vec<usize>,
        token_ind_of_table_name: usize,
        omit_csv_col_re: &str,
        omit_csv_row_re: &str,
        omit_big_row_re: &str,
    ) -> Builder {
        if is_verbose {
            println!(
                "Csv column delimiter is {}. Builder re mapping is {}",
                csv_col_del, builder_re_mapping
            );
        }
        let mut final_string = Builder::default();
        let mut row_reg = Regex::new(row_regex_str).unwrap();
        let mut index_1 = ind_1;
        let mut index_2 = ind_2;
        let mut split_row_by = split_r_by;
        let mut split_col_by = split_column_by;
        if is_struct {
            split_col_by = r"(Option<HashMap<String,Option<String>>>)|(String)|([_a-zA-Z0-9<>]+)";
            split_row_by = r"[\w\s\d,:\{<>--[\r\n\t]]{1,}(?:,)";
            index_1 = 1usize;
            index_2 = 2usize;
        }
        let mut rows: Vec<Match> = row_reg.find_iter(regexable_input).collect();
        if omit_big_row_re != "" {
            let re = Regex::new(omit_big_row_re);
            if let Ok(r) = re {
                rows = rows
                    .iter_mut()
                    .filter(move |big_row| {
                        let captures = r.captures(big_row.as_str());
                        if let Some(c) = captures {
                            return false;
                        } else {
                            return true;
                        }
                    })
                    .map(|t| *t)
                    .collect();
            } else {
                panic!(
                    "Could not create reg. expression from  --omit-big-row-re value: '{}'.",
                    omit_big_row_re
                );
            }
        }
        let row_size = rows.len();
        if row_size == 0 {
            panic!("No 'big' rows found.");
        }
        if is_verbose {
            dbg!(&csv_row_del);
        }
        let mut csv_rows: Vec<&str> = input.split(csv_row_del).collect();
        if is_verbose {
            dbg!(&csv_rows);
        }
        if !disable_assert_row_count && rows.len() != csv_rows.len() {
            panic!("'Big' row count is not the same as the count for csv rows.  Assertion failed!  Disable this feature with the -w flag.");
        }
        let mut rows_split: Vec<(Vec<String>, Vec<String>)> = Vec::new();
        let mut count = 0;
        'outer: for row in rows {
            //bigger rows
            for skip_val in &big_rows_to_skip {
                if *skip_val == count as usize {
                    continue 'outer;
                }
            }
            let (col_ind_1_list, col_ind_2_list) = retrieve_2_col_lists_from_rows_within_row(
                //same thing as (field_names, field_types)
                row.as_str(),
                split_row_by,
                split_col_by,
                index_1,
                index_2,
                is_verbose,
            );
            if is_verbose {
                println!(
                    "row: {}, col_ind_1: <{:?}>, col_ind_2: <{:?}>",
                    count, col_ind_1_list, col_ind_2_list
                );
            }
            rows_split.push((col_ind_1_list, col_ind_2_list));
            count += 1;
        }
        count = 0;
        if omit_csv_row_re != "" {
            csv_rows  = omit_rows_by_re(omit_csv_row_re, csv_rows);
        }
        for csv_row in csv_rows {
            let temp = rows_split.clone();
            let row_ind = get_usize_of_col_with_col_index_of_row(
                csv_row,
                csv_col_del,
                token_ind_of_table_name,
            );
            let (col_ind_1_list, col_ind_2_list) = temp.get(row_ind).unwrap();
            let mut modified_template = buildOutputFromEntry(
                csv_row.to_string(),
                &mut template.clone(),
                csv_col_del,
                token,
                trim_endlines,
                is_verbose,
                omit_csv_col_re,
            );
            if is_verbose {
                print!("{}\r", modified_template.as_str());
            }
            let col_1_size = col_ind_1_list.len();
            let col_2_size = col_ind_2_list.len();
            if col_1_size != col_2_size {
                panic!("The number of columns in col index 1 should have the same count as the number of columns in col index 2.  Assertion failed!  These values are set with the --col-1-index and --col-2-index, which default to 0 and 1 respectively.");
            }
            'outer: for i in 0..col_1_size {
                for x in &little_rows_to_skip {
                    if *x == i {
                        continue 'outer;
                    }
                }
                let col_1 = col_ind_1_list[i].clone();
                let col_2 = col_ind_2_list[i].clone();
                modified_template = modify_template_based_on_row(
                    col_1,
                    col_2,
                    modified_template.as_str(),
                    is_struct,
                    i == col_1_size - 1,
                );
                if is_verbose {
                    print!("{}\r", modified_template.as_str());
                }
            }
            /*let mut csv_row_indices: Vec<usize> = vec![other_index];
            if builder_re_mapping != -1 {
                csv_row_indices = retrieve_csv_row_indices_by_col_mapping_and_row(
                    &csv_rows,
                    &row,
                    builder_re_mapping,
                    csv_col_del,
                    is_verbose,
                );
            }
            if is_verbose {
                println!("Indices length: {}", csv_row_indices.len());
            }
            if is_verbose {
                println!(
                    "Csv-row indices are {:?} with bigger row index: {}",
                    csv_row_indices, other_index
                );
            }*/
            final_string.append(format!("{}\n", modified_template.as_str()));
        }
        final_string
    }
    pub fn get_usize_of_col_with_col_index_of_row(row: &str, col_del: &str, ind: usize) -> usize {
        let col = split_csv_row_into_vec(row, col_del);
        col[ind].parse::<usize>().unwrap()
    }
    pub fn retrieve_csv_row_indices_by_col_mapping_and_row(
        csv_rows: &Vec<&str>,
        input_row: &Match,
        csv_col_mapping: i16,
        col_del: &str,
        is_verbose: bool,
    ) -> Vec<usize> {
        let mut indices = Vec::new();
        let mut val = 0;
        if is_verbose {
            println!("Csv rows are: {:?}\n", csv_rows);
        }
        for csv_row in csv_rows {
            let cols = csv_row.split(col_del);
            let mut index = 0i16;
            'inner: for col in cols {
                if index == csv_col_mapping {
                    let mut builder = Builder::default();
                    builder.append(col);
                    let re = Regex::new(builder.string().unwrap().as_str()).unwrap();
                    match re.captures(input_row.as_str()) {
                        Some(success) => {
                            if is_verbose {
                                //println!("Matched input row, {}, to column. {:?}", col, success);
                            }
                            indices.push(val);
                            break 'inner;
                        }
                        None => {
                            break 'inner;
                        }
                    }
                }
                index += 1;
            }
            val += 1;
        }
        return indices;
    }
    fn retrieve_2_col_lists_from_rows_within_row<'b>(
        row: &str,
        split_row_by: &str,
        split_col_by: &str,
        index_1: usize,
        index_2: usize,
        is_verbose: bool,
    ) -> (Vec<String>, Vec<String>) {
        let row_regex = Regex::new(split_row_by).unwrap();
        let rows = row_regex.find_iter(row);
        let split_col_re = Regex::new(split_col_by).unwrap();
        let mut field_names = Vec::new();
        let mut field_types = Vec::new();
        for r in rows {
            let columns: Vec<Match> = split_col_re
                .find_iter(r.as_str())
                .filter(|s| -> bool {
                    if s.as_str() == "" {
                        return false;
                    } else {
                        return true;
                    }
                })
                .collect();
            if columns.len() > 1 {
                field_names.push(columns[index_1].as_str().to_string());
                field_types.push(columns[index_2].as_str().to_string());
            } else if is_verbose {
                panic!("{:?} was only {} in size!", columns, columns.len());
            }
        }
        (field_names, field_types)
    }
    pub fn modify_template_based_on_row(
        col_1: String,
        col_2: String,
        template: &str,
        is_struct: bool,
        is_last_row: bool,
    ) -> String {
        let mut modified_template = template.to_string();
        if is_struct {
            //then col_1 is column_name or variable name
            //then col_2 is column type
            let mut builder = Builder::default();
            // for col names separated by columns...
            if is_last_row {
                builder.append(col_1.clone());
            } else {
                builder.append(col_1.clone());
                builder.append(", %COLS%");
            }
            modified_template =
                modified_template.replacen("%COLS%", builder.string().unwrap().as_str(), 1000);
            // for format! placeholders in the SQL statement, where field type matters.  Separated by columns.
            builder = Builder::default();
            let mut x = col_2.contains("String");
            //let mut y = col_2.contains("DateTimeUtc");
            //let z = col_2.contains("NaiveDate");
            let isArray = col_2.contains("Vec");
            let isHash = col_2.contains("HashMap");
            builder.append("{}");
            if !is_last_row {
                builder.append(", %COL_PLACEMENTS%");
            }
            modified_template = modified_template.replacen(
                "%COL_PLACEMENTS%",
                builder.string().unwrap().as_str(),
                1000,
            );
            // for format! variables after the first parameter in order... default variable name is "obj."
            builder = Builder::default();
            x = col_2.contains("Option");
            if x {
                builder.append("deserialize_option(");
            } else {
                builder.append("deserialize_(&");
            }
            if isArray {
                builder.append("\nformat!(\"{:?}\",");
            } else if isHash {
                builder.append("unwrap_hashmap_into_string(&");
            }
            if is_last_row {
                builder.append("obj.");
                if x {
                    builder.append(col_1);
                    builder.append(".as_ref()");
                } else {
                    builder.append(col_1);
                }
                if isArray {
                    builder.append(r#").replacen(r"[", "{", 3000)"#);
                    builder.append(r#".replacen(r"]", "}", 3000)"#);
                } else if isHash {
                    builder.append(")");
                }
                builder.append(")");
            } else {
                builder.append("obj.");
                if x {
                    builder.append(col_1);
                    builder.append(".as_ref()");
                } else {
                    builder.append(col_1);
                }
                if isArray {
                    builder.append(r#").replacen(r"[", "{", 3000)"#);
                    builder.append(r#".replacen(r"]", "}", 3000)"#);
                } else if isHash {
                    builder.append(")");
                }
                builder.append(")");
                builder.append(", %OBJ_COLS%");
            }
            modified_template =
                modified_template.replacen("%OBJ_COLS%", builder.string().unwrap().as_str(), 1000);
            return modified_template;
        }
        "".to_string()
    }
    pub fn postBuild(file_name: &str, encoding: &str) -> String {
        let file = String::from_utf8(read_file(file_name, encoding).unwrap())
            .expect(format!("Failed to decode with {}.", encoding).as_str());
        let re = Regex::new(r"fn [a-z_]{1,}").unwrap();
        let mut functions = Vec::new();
        let mut index = 0;
        let iterz = re.find_iter(file.as_str());
        for capture in iterz {
            if capture.as_str().len() > 3 {
                let fn_name = &capture.as_str()[3..capture.as_str().len()];
                functions.push(fn_name);
            }
            index += 1;
        }
        let mut other_index = 0;
        let mut builder = Builder::default();
        for capture in functions {
            builder.append(capture);
            other_index += 1;
            if other_index == index - 1 {
                break;
            }
            builder.append(",");
        }
        let res = builder.string().unwrap();
        res
    }
}
