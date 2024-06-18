use crate::error::Error;
use serde_json::Value;
use std::{
    fs::File,
    io::{BufRead, BufReader}, sync::{Arc, Mutex}, thread,
};

pub fn get_keys_and_query(query: String) -> Result<(Vec<String>, Vec<String>), Error> {
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
        return Ok((query_parts, key_vec));
    }

    let mut stack: Vec<&str> = Vec::new();

    for i in 0..query_parts.len() {
        match query_parts[i].as_str() {
            "(" => {
                stack.push(&query_parts[i]);
            }
            ")" => {
                if let None = stack.pop() {
                    return Err(Error::InvalidBraces);
                }
            }
            "and" | "or" => {
                if query_parts[i+1] == ")" || query_parts[i+1]=="("{
                    return Err(Error::InvalidQuery);
                }
            }
            _ => {}
        }
        
        let temp_check = query_parts[i].split(':').collect::<Vec<&str>>()[0].to_string();
        if !key_vec.contains(&temp_check) {
            key_vec.push(temp_check);
        }
    }

    if !stack.is_empty() {
        return Err(Error::InvalidBraces);
    }

    Ok((key_vec, query_parts))
}

pub fn get_matched_lines(
    file_path: String,
    keys: &Vec<String>,
    query_vec: &Vec<String>,
) -> Result<Vec<Value>, Error> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(Error::FileOpening),
    };

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<_, _>>()
        .map_err(|_| Error::ReadLine)?;

    let num_threads = 4; 
    let chunk_size = (lines.len() + num_threads - 1) / num_threads; 

    let keys = Arc::new(keys.clone());
    let query_vec = Arc::new(query_vec.clone());
    let lines = Arc::new(lines);

    let mut handles = vec![];
    let result_vec = Arc::new(Mutex::new(Vec::new()));

    for i in 0..num_threads {
        let keys = Arc::clone(&keys);
        let query_vec = Arc::clone(&query_vec);
        let lines = Arc::clone(&lines);
        let result_vec = Arc::clone(&result_vec);

        let handle = thread::spawn(move || {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, lines.len());

            for line in &lines[start..end] {
                if let Ok(json_obj) = serde_json::from_str::<Value>(line) {
                    let mut temp_str = String::new();
                    for key in &*keys {
                        temp_str.push_str(key);
                        temp_str.push(':');
                        if let Some(value) = json_obj.get(key) {
                            temp_str.push_str(&value.to_string().trim_matches('"'));
                        }
                    }

                    let (result, _) = complete_check(&query_vec, &temp_str, &mut 0);
                    if result {
                        result_vec.lock().unwrap().push(json_obj);
                    }
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let res_vec = Arc::try_unwrap(result_vec).unwrap().into_inner().unwrap();
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
