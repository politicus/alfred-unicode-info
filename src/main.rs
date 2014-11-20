#![feature(slicing_syntax,globs)]

extern crate alfred;

use std::char;
use std::os;
use std::num;
use std::io;

mod icu;

static VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/version"));

fn main() {
    let args = os::args();

    let text = args[].get(1).map(|s| s[]).unwrap_or("");
    let _ = handle_arg(text);
}

/// Handles the given arg
fn handle_arg(text: &str) -> io::IoResult<()> {
    if text.is_empty() {
        return handle_placeholder();
    } else if text.starts_with("U+") && text.len() > 2 && text.len() <= 10 {
        let digits = text.slice_from(2);
        match num::from_str_radix::<u32>(digits, 16) {
            None => (),
            Some(code) => {
                // this is a U+#### codepoint
                if try!(handle_codepoint(code)) {
                    return Ok(());
                }
            }
        }
    }
    handle_text(text)
}

/// Prints out the XML for the given codepoint, if valid.
/// Returns `Ok(true)` if the codepoint is valid, `Ok(false)` if not.
fn handle_codepoint(code: u32) -> io::IoResult<bool> {
    let name = match icu::u_charName(code, icu::U_EXTENDED_CHAR_NAME) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(io::stderr(), "u_charName error: {}", e);
            return Ok(false);
        }
    };
    let name = match name[] {
        "" => "<unknown>",
        s => s
    };

    let mut xmlw = try!(alfred::XMLWriter::new(io::stdout()));

    let arg = char::from_u32(code).unwrap_or('\uFFFD').to_string();
    let title = format!("\u200B{}", arg);
    let subtitle = format!("U+{:04X} {}", code, name);

    let item = alfred::ItemBuilder::new(title)
                                   .arg(arg)
                                   .subtitle(subtitle)
                                   .icon_path("icon.png")
                                   .into_item();
    try!(xmlw.write_item(&item));

    let mut stdout = try!(xmlw.close());
    try!(stdout.flush());
    Ok(true)
}

/// Prints out the XML for the sequence of characters.
fn handle_text(text: &str) -> io::IoResult<()> {
    let mut xmlw = try!(alfred::XMLWriter::new(io::stdout()));

    for c in text.chars() {
        let name = match icu::u_charName(c as u32, icu::U_EXTENDED_CHAR_NAME) {
            Ok(s) => s,
            Err(e) => {
                let _ = writeln!(&mut io::stderr(), "u_charName error: {}", e);
                continue;
            }
        };
        let name = match name[] {
            "" => "<unknown>",
            s => s
        };
        let item = alfred::ItemBuilder::new(name)
                                       .arg(format!("U+{:04X} {}", c as u32, name))
                                       .subtitle(format!("U+{:04X}", c as u32))
                                       .icon_path("icon.png")
                                       .into_item();
        try!(xmlw.write_item(&item));
    }

    let mut stdout = try!(xmlw.close());
    try!(stdout.flush());
    Ok(())
}

/// Prints the placeholder item
fn handle_placeholder() -> io::IoResult<()> {
    let item = alfred::ItemBuilder::new("Unicode info for …")
                                   .subtitle(format!("version {}", VERSION))
                                   .valid(false)
                                   .into_item();
    alfred::write_items(io::stdout(), &[item])
}