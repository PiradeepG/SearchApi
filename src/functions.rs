use crate::error::Error;
use serde_json::Value;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub fn get_keys_and_query(query: String) -> (Vec<String>, Vec<String>) {
    let query_parts: Vec<String> = query
        .split_whitespace()
        .map(|x| x.trim_matches('"').to_string())
        .collect();
    let mut key_vec: Vec<String> = Vec::new();

    if query_parts.len() == 1 {
        key_vec.push(
            query_parts[0].split(':').collect::<Vec<&str>>()[0]
                .trim_matches('"')
                .to_string(),
        );
        return (query_parts, key_vec);
    }

    for i in &query_parts {
        if i == "and" || i == "or" || i == "(" || i == ")" {
            continue;
        }

        let temp_check = i.split(':').collect::<Vec<&str>>()[0].to_string();
        if !key_vec.contains(&temp_check) {
            key_vec.push(temp_check);
        }
    }
    (key_vec, query_parts)
}

pub fn get_matched_lines(
    file_path: String,
    keys: &Vec<String>,
    query_vec: &Vec<String>,
) -> Result<Vec<Value>, Error> {
    let mut res_vec: Vec<Value> = Vec::new();
    let file = match File::open(file_path) {
        Ok(msg) => msg,
        Err(_) => return Err(Error::FileOpening),
    };

    let reader = BufReader::new(file);

    for line in reader.lines() {
        let json_str = match line {
            Ok(msg) => msg,
            Err(_) => return Err(Error::ReadLine),
        };

        let mut temp_str: String = String::new();
        let json_obj: Value = match serde_json::from_str::<Value>(&json_str) {
            Ok(msg) => {
                for key in keys {
                    temp_str.push_str(key);
                    temp_str.push(':');

                    match msg.get(key) {
                        Some(value) => {
                            temp_str.push_str(&value.to_string().trim_matches('"'));
                        }
                        None => {}
                    }
                }
                msg
            }
            Err(_) => return Err(Error::JsonParsing),
        };
        let (result, _) = complete_check(query_vec, &temp_str, &mut 0);
        
        if result {
            res_vec.push(json_obj);
        }
    }
    Ok(res_vec)
}

fn complete_check(query_vec: &Vec<String>, line: &String, index: &mut usize) -> (bool, usize) {
    let mut temp = true;
    while *index < query_vec.len() {
        let temp_check = &query_vec[*index];

        match temp_check.as_str() {
            "and" => {
                if temp {
                    *index += 1;
                } else {
                    if !(&query_vec[*index + 1] == "(") {
                        *index += 2;
                    } else {
                        let mut stack: Vec<&String> = Vec::new();
                        let mut next_variable;
                        loop {
                            next_variable = &query_vec[*index];
                            *index += 1;
                            if next_variable == "(" {
                                stack.push(next_variable);
                            }
                            if next_variable == ")" {
                                if let Some(_) = stack.last() {
                                    stack.pop();
                                }
                            }
                            if stack.is_empty() {
                                break;
                            }
                        }
                        *index += 1;
                    }
                }
            }
            "or" => {
                if temp {
                    if !(&query_vec[*index + 1] == "(") {
                        *index += 2;
                    } else {
                        let mut stack: Vec<&String> = Vec::new();
                        let mut next_variable;
                        loop {
                            next_variable = &query_vec[*index];
                            *index += 1;
                            if next_variable == "(" {
                                stack.push(next_variable);
                            }
                            if next_variable == ")" {
                                if let Some(_) = stack.last() {
                                    stack.pop();
                                }
                            }
                            if stack.is_empty() {
                                break;
                            }
                        }
                        *index += 1;
                    }
                } else {
                    *index += 1;
                }
            }
            "(" => {
                *index += 1;

                let (inner_temp, new_index) = complete_check(query_vec, line, index);
                temp = inner_temp;
                *index = new_index;
            }
            ")" => {
                *index += 1;
                return (temp, *index);
            }
            _ => {
                temp = line.contains(temp_check);
                *index += 1;
            }
        }
    }
    (temp, *index)
}
