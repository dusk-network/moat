use std::fs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "cdef.pest"]
pub struct CDefParser;

fn main() {
    let unparsed_file = fs::read_to_string("country_age.cdef").expect("circuit definition file read successfully");
    let file = CDefParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next().unwrap(); // get and unwrap the `file` rule; never fails

    for property in file.into_inner() {
        match property.as_rule() {
            Rule::property => {
                println!("{}", property);
            }
            Rule::EOI => (),
                _ => unreachable!(),
        }
    }

}
