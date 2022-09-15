use std::env;

use zaffre::font::{self, FontWeight, FontWidth, Font};
use zaffre::text::TextAnalyzer;

extern crate zaffre;

fn print_analysis(string: &str, font: &Font) {
    let mut analyzer = TextAnalyzer::new();
    analyzer.set_text_from(&string.to_owned());
    println!("{}", analyzer.text());
    for r in analyzer.get_runs() {
       let glyph_run = analyzer.get_glyphs_and_positions(r.text_range(), r.clone(), &font);
       println!("{:#?}", glyph_run);
    }
    println!();
}

fn main() {
    // let font = font::get_matching_font(
    //     "Arial", FontWeight::Normal, font::FontSlant::Normal, FontWidth::Normal);
    let deja_vu_family = font::get_family("DejaVu Sans")
        .or_else(|| font::get_family("Helvetica"))
        .or_else(|| font::get_family("Arial"))
        .expect("couldn't find font");
    let deja_vu_font = deja_vu_family.get_styles()[0].get_font(15.0);
    let noto_devan_family = font::get_family("Noto Sans Devanagari")
        .or_else(|| font::get_family("Helvetica"))
        .or_else(|| font::get_family("Arial"))
        .expect("couldn't find font");
    let noto_devan_font = noto_devan_family.get_styles()[0].get_font(15.0);

    print_analysis("First, a plain English sentence.", &deja_vu_font);
    print_analysis("A 🍕pizza", &deja_vu_font);
    print_analysis("Then, свобода, as a test", &deja_vu_font);
    print_analysis("Something including a עברית word.", &deja_vu_font);
    print_analysis("काकि", &noto_devan_font);
    // TODO: one sample with multiple bidi levels

    let mut args = env::args();
    args.next().unwrap();
    let custom_string = args.next();
    if let Some(custom_string) = custom_string {
        print_analysis(&custom_string, &deja_vu_font);
    }
}