pub mod error;
pub mod functions;
use crate::error::Error;
use functions::*;
use serde_json::Value;
pub fn search_query(file_path: String, query_string: String) -> Result<Vec<Value>, Error> {
    let (keys, query_vec) = get_keys_and_query(query_string.trim_end().to_string())?;

    Ok(get_matched_lines(file_path, &keys, &query_vec))?
}

fn main() {
    match search_query(
        String::from("/home/piradeep/Downloads/sample_data_mixed_lines.json"),
        String::from("class:10"),
    ) {
        Ok(msg) => println!("{:#?}", msg),
        Err(err) => println!("{:?}", err),
    };
}
