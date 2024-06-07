#[derive(Debug)]
pub enum Error{
    FileOpening,
    ReadLine,
    JsonParsing,
    GettingOutput,
}