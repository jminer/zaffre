use std::env;

use zaffre::text::TextAnalyzer;

extern crate zaffre;

fn print_analysis(string: &str) {
    let analyzer = TextAnalyzer::new(string.to_owned());
    println!("{}", analyzer.text());
    for r in analyzer.get_runs() {
        println!("{:#?}", r);
    }
    println!();
}

fn main() {
    print_analysis("First, a plain English sentence.");
    print_analysis("Then, свобода, as a test");
    print_analysis("Something including a עברית word.");
    // TODO: one sample with multiple bidi levels

    let mut args = env::args();
    args.next().unwrap();
    let custom_string = args.next();
    if let Some(custom_string) = custom_string {
        print_analysis(&custom_string);
    }
}