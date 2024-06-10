use std::{fs::File, io::{BufRead, BufReader}};
use serde_json::Value;
use crate::error::Error;


pub fn get_keys_and_query(query: String) -> (Vec<String>, Vec<String>) {
    let query_parts: Vec<String> = query.split_whitespace().map(|x| x.trim_matches('"').to_string()).collect();
    let mut key_vec: Vec<String> = Vec::new();

    if query_parts.len() == 1{
        key_vec.push(query_parts[0].split(':').collect::<Vec<&str>>()[0].trim_matches('"').to_string());
        return  (query_parts, key_vec)
    }

    for i in (0..query_parts.len()).step_by(2){
        let temp_check=query_parts[i].split(':').collect::<Vec<&str>>()[0].to_string();
        if !key_vec.contains(&temp_check){
            key_vec.push(temp_check);
        }
    }
    (key_vec,query_parts)
}

pub fn get_matched_lines(file_path:String,keys:&Vec<String>,query_vec:&Vec<String>)->Result<Vec<Value>,Error>{

    let mut res_vec:Vec<Value>=Vec::new();
    let file = match File::open(file_path){
        Ok(msg) => msg,
        Err(_) => return Err(Error::FileOpening),
    };

    let reader = BufReader::new(file);

    for line in reader.lines(){
        let json_str = match line{
            Ok(msg) => msg,
            Err(_) => return Err(Error::ReadLine),
        };

        let mut temp_str:String=String::new();
        let json_obj:Value=match serde_json::from_str::<Value>(&json_str) {
            Ok(msg) => {
                for key in keys {
                    temp_str.push_str(key);
                    temp_str.push(':');
                    
                    match msg.get(key) {
                        Some(value) => {
                            temp_str.push_str(&value.to_string().trim_matches('"'));
                        },
                        None => {},
                    }
                }
                msg
            },
            Err(_) => return Err(Error::JsonParsing),
        };
        if is_matching(query_vec, temp_str){
            res_vec.push(json_obj);
        }
    }
    Ok(res_vec)
}


pub fn is_matching(query_vec:&Vec<String>,line:String)->bool{
    let mut temp=true;
    let mut i=0;

    while i < query_vec.len(){
        let temp_check=&query_vec[i];
        match temp_check.as_str(){
            "and" => {
                if temp {
                    i += 1;
                } else {
                    i += 2;
                }
                continue;
            },
            "or" => {
                if temp {
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            },
            _ => {
                temp = line.contains(temp_check);
                i+=1;
            }
        }
    }
    temp
}