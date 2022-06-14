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
        println!("{}", family.get_family_name());
        for style in family.get_styles() {
            println!("  {:17} weight: {}, slant: {:?}, monospaced: {}",
                style.get_style_name(), style.weight().0, style.slant(), style.is_monospaced());
        }
        println!();
    }
}

