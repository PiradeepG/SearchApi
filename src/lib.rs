pub mod error;
pub mod functions;
use crate::error::Error;
use functions::*;
use serde_json::Value;
pub fn search_query(file_path:String,query_string:String)-> Result<Vec<Value>,Error>{

    let (keys,query_vec)=get_keys_and_query(query_string);

    Ok(get_matched_lines(file_path, &keys, &query_vec))?
}
