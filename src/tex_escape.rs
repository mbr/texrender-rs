//! LaTeX-escaping.
//!
//! Contains functions for escaping text into (La)TeX documents.

use std::io;
use std::io::Write;

/// Escapes a string for use in TeX document and writes it out.
pub fn write_escaped<W>(mut out: W, string: &str) -> io::Result<()>
where
    W: Write,
{
    for c in string.chars() {
        match c {
            '&' => out.write_all(b"\\&")?,
            '%' => out.write_all(b"\\%")?,
            '$' => out.write_all(b"\\$")?,
            '#' => out.write_all(b"\\#")?,
            '_' => out.write_all(b"\\_")?,
            '{' => out.write_all(b"\\{")?,
            '}' => out.write_all(b"\\}")?,
            '~' => out.write_all(b"\\textasciitilde{}")?,
            '^' => out.write_all(b"\\textasciicircum{}")?,
            '\\' => out.write_all(b"\\textbackslash{}")?,

            // Potentially optional, but likely good practice nonetheless.
            '<' => out.write_all(b"\\textless{}")?,
            '>' => out.write_all(b"\\textgreater{}")?,
            '|' => out.write_all(b"\\textbar{}")?,
            '"' => out.write_all(b"\\textquotedbl{}")?,

            // Prevent issues with '\\' linebreaks.
            '[' => out.write_all(b"{[}")?,
            ']' => out.write_all(b"{]}")?,

            // Everything else passes through unscathed.
            _ => write!(out, "{}", c)?,
        }
    }

    Ok(())
}
