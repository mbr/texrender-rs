//! Predefined LaTeX elements.
//!
//! Contains a number of elements that can be used to generate LaTeX code. See the `tpl` module
//! documentation for an example and comparison.

use super::{Args, BeginEndBlock, Group, MacroCall, OptArgs, RawTex, TableRow, TexElement, Text};

/// A no-item iterator.
///
/// Can be passed to `Args::new` or `OptArgs::new` to indicate no arguments.
#[derive(Copy, Clone, Debug)]
pub struct Nothing;

impl Iterator for Nothing {
    type Item = String;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

/// Ready to use instance of `Nothing`.
pub const N: Nothing = Nothing;

/// Creates a new top-level document.
#[inline]
pub fn doc(children: Vec<Box<dyn TexElement>>) -> Group {
    Group(children)
}

/// Creates a `documentclass` declaration.
#[inline]
pub fn documentclass<S: AsRef<str>, I: IntoIterator<Item = S>>(
    opt_args: I,
    doc_class: &str,
) -> MacroCall {
    MacroCall::new(
        RawTex::new("documentclass"),
        OptArgs::new(opt_args),
        Args::new(&[doc_class]),
    )
}

/// Creates a new `document` environment.
#[inline]
pub fn document(children: Vec<Box<dyn TexElement>>) -> BeginEndBlock {
    BeginEndBlock::new("document", OptArgs::default(), Args::default(), children)
}

/// Creates a new `figure` environment.
pub fn figure(options: &str, children: Vec<Box<dyn TexElement>>) -> BeginEndBlock {
    BeginEndBlock::new(
        "figure",
        OptArgs::new(&[options]),
        Args::default(),
        children,
    )
}

/// Creates an anonymous group.
#[inline]
pub fn group(children: Vec<Box<dyn TexElement>>) -> Group {
    Group(children)
}

/// Creates a `hspace` element.
#[inline]
pub fn hspace<S: Into<String>>(space: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("hspace"),
        opt_args: OptArgs::new(N),
        args: Args(vec![Box::new(RawTex::new(space)) as Box<dyn TexElement>]),
        newline: true,
    }
}

/// Creates a new `minipage` environment.
pub fn minipage(alignment: &str, width: &str, children: Vec<Box<dyn TexElement>>) -> BeginEndBlock {
    BeginEndBlock::new(
        "minipage",
        OptArgs::new(&[alignment]),
        Args::new(&[width]),
        children,
    )
}

/// Creates new, unescaped LaTeX-code.
#[inline]
pub fn raw<S: Into<String>>(raw: S) -> RawTex {
    RawTex::new(raw)
}

/// Creates a new `section` header.
#[inline]
pub fn section<S: Into<String>>(title: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("section"),
        opt_args: OptArgs::default(),
        args: Args(elems![t(title)]),
        newline: true,
    }
}

/// Creates a new `subsection` header.
#[inline]
pub fn subsection<S: Into<String>>(title: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("subsection"),
        opt_args: OptArgs::default(),
        args: Args(elems![t(title)]),
        newline: true,
    }
}

/// Creates a new text element.
#[inline]
pub fn t<S: Into<String>>(s: S) -> Text {
    Text::new(s)
}

/// Creates a row in a table.
#[inline]
pub fn table_row(cols: Vec<Box<dyn TexElement>>) -> TableRow {
    TableRow::new(cols)
}

/// Creates a new `tabular` environment.
#[inline]
pub fn tabular<S: Into<String>, T: Into<String>>(
    width: T,
    column_definitions: S,
    children: Vec<Box<dyn TexElement>>,
) -> BeginEndBlock {
    BeginEndBlock::new(
        "tabular",
        OptArgs::new(N),
        Args::new_from_elements(elems!(raw(width), raw(column_definitions))),
        children,
    )
}

/// Creates a new `tabularx` environment.
#[inline]
pub fn tabularx<S: Into<String>, T: Into<String>>(
    width: T,
    column_definitions: S,
    children: Vec<Box<dyn TexElement>>,
) -> BeginEndBlock {
    BeginEndBlock::new(
        "tabularx",
        OptArgs::new(N),
        Args::new_from_elements(elems!(raw(width), raw(column_definitions))),
        children,
    )
}

/// Creates a new `textbf` element.
#[inline]
pub fn textbf<T: TexElement + 'static>(inner: T) -> MacroCall {
    MacroCall::new_inline(
        RawTex::new("textbf"),
        OptArgs::new(N),
        Args::new_from_elements(vec![Box::new(inner) as Box<dyn TexElement>]),
    )
}

/// Creates a `usepackage` declaration.
#[inline]
pub fn usepackage<S: Into<String>>(
    opt_args: Vec<Box<dyn TexElement>>,
    package_name: S,
) -> MacroCall {
    MacroCall {
        ident: RawTex::new("usepackage"),
        opt_args: OptArgs(opt_args),
        args: Args(vec![
            Box::new(RawTex::new(package_name)) as Box<dyn TexElement>
        ]),
        newline: true,
    }
}

/// Creates a `vspace` element.
#[inline]
pub fn vspace<S: Into<String>>(space: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("vspace"),
        opt_args: OptArgs::new(N),
        args: Args(vec![Box::new(RawTex::new(space)) as Box<dyn TexElement>]),
        newline: true,
    }
}
