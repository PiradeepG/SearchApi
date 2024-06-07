pub mod error;
pub mod query_checker;
use query_checker::get_matched_lines;
fn main(){

    match get_matched_lines(String::from("/home/piradeep/Downloads/1_Untitled.json"),String::from("_zl_timestamp=\"1717521124998\" and message=\"[src/pre_start/pre_population.rs:17] pre populating sandboxes network config\"")){
        Ok(msg) => println!("{:?}",msg),
        Err(msg) => println!("{:?}",msg),
    };

} 
