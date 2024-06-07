use serde_json::Value;
use std::{fs::File, io::{BufRead, BufReader}};
use crate::error::Error;


pub fn get_matched_lines(file_path:String,query:String)->Result<Vec<Value>, Error>{

    let file = match File::open(file_path){
        Ok(msg) => msg,
        Err(_) => return Err(Error::FileOpening),
    };

    let reader = BufReader::new(file);

    let mut result_vec: Vec<Value> = Vec::new();

    for line in reader.lines() {
        let json_str = match line{
            Ok(msg) => msg,
            Err(_) => return Err(Error::ReadLine),
        };

        match serde_json::from_str(&json_str) {
            Ok(msg) => {
                // println!("{:?}",msg);
                if is_matching(&msg, &query) {
                    result_vec.push(msg);
                }
            },
            Err(_) => return Err(Error::JsonParsing),
        } 
        
    }

    Ok(result_vec)

}

fn is_matching(item:&Value,query:&str) -> bool{

    let conditions:Vec<&str>=query.split(" and ").collect();

    for c in conditions{
        if c.contains(" or "){
            let sub_conditions:Vec<&str>=c.split(" or ").collect();

            if sub_conditions.iter().any(|sub_c| is_matching(item, sub_c)){
                return true;
            }
        }
        else{
            let last_parts:Vec<&str>=c.split("=").map(|s| s.trim_matches('"')).collect();

            if let (Some(key),Some(value))=(last_parts.get(0),last_parts.get(1)){
                if let Some(actual_value) = item.get(key) {
                    match actual_value {
                        Value::String(actual_value_str) => {
                            if actual_value_str == value {
                                return true;
                            }
                        }
                        Value::Number(actual_value_num) => {
                            if let Ok(value_num) = value.parse::<serde_json::Number>() {
                                if actual_value_num == &value_num {
                                    return true;
                                }
                            }
                        }
                        Value::Bool(actual_value_bool) => {
                            if let Ok(value_bool) = value.parse::<bool>() {
                                if actual_value_bool == &value_bool {
                                    return true;
                                }
                            }
                        }
                        _ => {

                        }
                    }
                }
            } 
        }
    }
    false
}