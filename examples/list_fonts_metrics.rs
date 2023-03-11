/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 */

extern crate zaffre;

use zaffre::font;
use zaffre::{Point2, Size2};

fn main() {
    let font_families = font::get_families();
    for family in font_families {
        for style in family.get_styles() {
            println!("** {} {} **", family.get_family_name(), style.get_style_name());
            let font = style.get_font(20.0);
            let metrics = font.metrics();
            println!("{:#?}", metrics);
        }
        println!();
    }
}

