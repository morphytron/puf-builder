pub mod builder {
    use regex::{Match, Regex};
    use crate::io::io::{read_file, write_file};
    use string_builder::Builder;
    use std::sync::{Mutex, MutexGuard};

    pub fn buildOutputFromEntry(
        entry: &String,
        template: String,
        col_del: &str,
        tolken: &str,
        trim_endlines: bool,
    ) -> String {
        let mut cols = entry.split(col_del);
        let (size, optional_size) = cols.size_hint();
        let mut index = 1;
        let mut clone = template.clone();
        for mut col in cols {
            let mut tolken_replacement = format!("{0}{1}", tolken.to_string(), index.to_string());
            if trim_endlines {
                let col_size = tolken_replacement.len();
                tolken_replacement = tolken_replacement
                    .as_str()
                    .trim_right_matches("\r")
                    .to_string();
                tolken_replacement = tolken_replacement.trim_right_matches("\n").to_string();
            }
            clone = clone.replace(&tolken_replacement, col);
            //println!("{}",clone);
            index += 1;
        }
        clone
    }

    pub fn buildOutput(
        input: &String,
        template: String,
        col_del: &str,
        row_del: &str,
        tolken: &str,
        trim_endlines: bool,
    ) -> String {
        let mut mut_input = input;
        let mut builder = Builder::default();
        let mut rows = input.as_str().split(row_del);
        for row in rows {
            let mut cols = row.split(col_del);
            let (size, optional_size) = cols.size_hint();
            let mut index = 1;
            let mut clone = template.clone();
            for mut col in cols {
                let mut tolken_replacement =
                    format!("{0}{1}", tolken.to_string(), index.to_string());
                if trim_endlines {
                    let col_size = tolken_replacement.len();
                    tolken_replacement = tolken_replacement
                        .as_str()
                        .trim_right_matches("\r")
                        .to_string();
                    tolken_replacement = tolken_replacement.trim_right_matches("\n").to_string();
                }
                clone = clone.replace(&tolken_replacement, col);
                //println!("{}",clone);
                index += 1;
            }
            builder.append(clone);
        }
        let s = builder.string().unwrap();
        s
    }

    pub fn split_input_into_row_set(
        input: &str,
        csv_row_del: &str,
        csv_col_del: &str,
        tolken: &str,
        trim_endlines: bool,
        regexable_input: &str,
        ind_1: usize,
        ind_2: usize,
        row_regex_str: &str,
        split_r_by: &str,
        split_column_by: &str,
        is_struct: bool,
        template: &str,
        is_verbose: bool,
        builder_re_mapping: i16,
    ) -> Builder {
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
        let mut rows: Vec<_> = row_reg.find_iter(regexable_input).collect();
        let mut rows_clone: Vec<_> = row_reg.find_iter(regexable_input).collect();
        //let mut sections : Vec<&str> = Vec::new();
        let row_size = rows.len();
        if is_struct && row_size == 0 {
            panic!("No pub structs found.");
        }
        let mut csv_rows: Vec<&str> = input.split(csv_row_del).collect();
        let mut other_index = 0;
        for row in rows {
            let mut modified_template = String::new();
            template.clone_into(&mut modified_template);
            //sections.push(row);
            if is_verbose {
                println!("Input row is: {}\n", row.as_str());
            }
            let (field_names, field_types) = retrieve_2_col_lists_from_rows_within_row(
                row.as_str(),
                split_row_by,
                split_col_by,
                index_1,
                index_2,
                is_verbose,
            );
            if is_verbose {
                println!(
                    "field_names: <{:?}>, field_types: <{:?}>",
                    field_names, field_types
                );
            }
            let size = field_names.len();
            let type_s = field_types.len();
            if size != type_s {
                panic!("Field-types-array size does not match field-names-array size.");
            }
            if is_verbose {
                println!(
                    "Csv col del is {}. Builder re mapping is {}",
                    csv_col_del, builder_re_mapping
                );
            }
            let mut csv_row_indices: Vec<usize> = vec![other_index];
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
                println!(
                    "Csv-row indices are {:?} with bigger row index: {}",
                    csv_row_indices, other_index
                );
            }
            for index in csv_row_indices {
                for i in 1..size {
                    let col_1 = field_names[i].clone();
                    let col_2 = field_types[i].clone();
                    modified_template = modify_template_based_on_row(
                        col_1,
                        col_2,
                        modified_template.as_str(),
                        is_struct,
                        i == size - 1,
                    );
                    modified_template = buildOutputFromEntry(
                        &csv_rows[index].to_string(),
                        modified_template.to_string(),
                        csv_col_del,
                        tolken,
                        trim_endlines,
                    );
                }
                final_string.append(format!("{}\n", modified_template));
            }
            other_index += 1;
        }
        final_string
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
        for csv_row in csv_rows {
            let cols = csv_row.split(col_del);
            let mut index = 0i16;
            'inner: for col in cols {
                if index == csv_col_mapping {
                    //if is_verbose {
                    //    println!("Csv row: {}", csv_row);
                    //    println!("Col: {}\n  ", col);
                    //}
                    let mut builder = Builder::default();
                    builder.append(col);
                    let re = Regex::new(builder.string().unwrap().as_str()).unwrap();
                    match re.captures(input_row.as_str()) {
                        Some(success) => {
                            println!("Matched input row to column.  {:?}", success);
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
            if is_verbose {
                println!("Smaller row is {}, columns is {:?}", r.as_str(), columns);
            }
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
            let mut y = col_2.contains("NaiveDateTime");
            let z = col_2.contains("NaiveDate");
            let isArray = col_2.contains("Vec");
            let isHash = col_2.contains("HashMap");
            if x || y || z {
                builder.append("'");
            }
            if isArray {
                builder.append("{:?}");
            } else {
                builder.append("{}");
            }
            if is_last_row {
                if x || y || z {
                    builder.append("'");
                }
            } else {
                if x || y || z {
                    builder.append("'");
                }
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
            if isArray {
                builder.append("\nformat!(\"{:?}\",");
            } else if isHash {
                builder.append("unwrap_hashmap_into_string(&");
            }
            if is_last_row {
                builder.append("obj.");
                if x {
                    builder.append(col_1);
                    builder.append(".as_ref().unwrap()");
                } else {
                    builder.append(col_1);
                }
                if isArray {
                    builder.append(r#").replacen(r"[", "{", 3000)"#);
                    builder.append(r#".replacen(r"]", "}", 3000)"#);
                } else if isHash {
                    builder.append(")");
                }
            } else {
                builder.append("obj.");
                if x {
                    builder.append(col_1);
                    builder.append(".as_ref().unwrap()");
                } else {
                    builder.append(col_1);
                }
                if isArray {
                    builder.append(r#").replacen(r"[", "{", 3000)"#);
                    builder.append(r#".replacen(r"]", "}", 3000)"#);
                }
                builder.append(", %OBJ_COLS%");
            }
            modified_template =
                modified_template.replacen("%OBJ_COLS%", builder.string().unwrap().as_str(), 1000);
            return modified_template;
        }
        "".to_string()
    }

    pub fn postBuild(file_name: &str, encoding: &str) -> String {
        let file =
            String::from_utf8(read_file(file_name, encoding).unwrap())
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
        //println!("{}", &res);
        res
    }
}
