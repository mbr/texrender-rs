//! Predefined LaTeX elements.
//!
//! Contains a number of elements that can be used to generate LaTeX code. See the `tpl` module
//! documentation for an example and comparison.

use super::{
    Args, BeginEndBlock, Group, IntoTexElement, MacroCall, OptArgs, RawTex, TableRow, TexElement,
};

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

/// Creates a new cell-coloring instruction (from the `colorx` package).
pub fn cellcolor<S: Into<String>>(color: S) -> MacroCall {
    MacroCall::new("cellcolor", OptArgs::default(), Args::single(raw(color)))
}

/// Creates a new top-level document.
#[inline]
pub fn doc(children: Vec<Box<dyn TexElement>>) -> Group {
    Group(children)
}

/// Creates a `documentclass` declaration.
#[inline]
pub fn documentclass<T: IntoTexElement>(
    opt_args: Vec<Box<dyn TexElement>>,
    doc_class: T,
) -> MacroCall {
    MacroCall::new(
        "documentclass",
        OptArgs::new(opt_args),
        Args::single(doc_class),
    )
}

/// Creates a new `document` environment.
#[inline]
pub fn document(children: Vec<Box<dyn TexElement>>) -> BeginEndBlock {
    BeginEndBlock::new("document", OptArgs::default(), Args::default(), children)
}

/// Creates a new footnote.
pub fn footnote<E: IntoTexElement>(footnote_content: E) -> MacroCall {
    MacroCall::new(
        "footnote",
        OptArgs::default(),
        Args::single(footnote_content),
    )
}

/// Creates a new `figure` environment.
pub fn figure<T: IntoTexElement>(
    alignment: T,
    children: Vec<Box<dyn TexElement>>,
) -> BeginEndBlock {
    BeginEndBlock::new(
        "figure",
        OptArgs::single(alignment),
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
pub fn hspace<T: IntoTexElement>(space: T) -> MacroCall {
    MacroCall::new_inline("hspace", OptArgs::default(), Args::single(space))
}

/// Creates an `includegraphics` element.
#[inline]
pub fn includegraphics<T: IntoTexElement>(options: Vec<Box<dyn TexElement>>, path: T) -> MacroCall {
    MacroCall::new_inline("includegraphics", OptArgs::new(options), Args::single(path))
}

/// Creates a new `minipage` environment.
#[inline]
pub fn minipage<T: IntoTexElement, U: IntoTexElement>(
    alignment: T,
    width: U,
    children: Vec<Box<dyn TexElement>>,
) -> BeginEndBlock {
    BeginEndBlock::new(
        "minipage",
        OptArgs::single(alignment),
        Args::single(width),
        children,
    )
}

/// Creates an "empty" element, representing nothing.
pub fn nothing() -> impl IntoTexElement {
    ""
}

/// Creates new, unescaped LaTeX-code.
#[inline]
pub fn raw<S: Into<String>>(raw: S) -> RawTex {
    RawTex::new(raw.into().into_bytes())
}

/// Creates a new `section` header.
#[inline]
pub fn section<T: IntoTexElement>(title: T) -> MacroCall {
    MacroCall::new("section", OptArgs::default(), Args::single(title))
}

/// Creates a new `subsection` header.
#[inline]
pub fn subsection<T: IntoTexElement>(title: T) -> MacroCall {
    MacroCall::new("subsection", OptArgs::default(), Args::single(title))
}

/// Creates a row in a table.
#[inline]
pub fn table_row(cols: Vec<Box<dyn TexElement>>) -> TableRow {
    TableRow::new(cols)
}

/// Creates a new `tabular` environment.
///
/// Keep in mind that when passing column definitions, these should likely be passed as `raw`
/// values, otherwise potentially contained `|` will be escaped.
#[inline]
pub fn tabular<T: IntoTexElement, U: IntoTexElement>(
    width: T,
    column_definitions: U,
    children: Vec<Box<dyn TexElement>>,
) -> BeginEndBlock {
    BeginEndBlock::new(
        "tabular",
        OptArgs::default(),
        Args::new(vec![
            width.into_tex_element(),
            column_definitions.into_tex_element(),
        ]),
        children,
    )
}

/// Creates a new `tabularx` environment.
///
/// Keep in mind that when passing column definitions, these should likely be passed as `raw`
/// values, otherwise potentially contained `|` will be escaped.
#[inline]
pub fn tabularx<T: IntoTexElement, U: IntoTexElement>(
    width: T,
    column_definitions: U,
    children: Vec<Box<dyn TexElement>>,
) -> BeginEndBlock {
    BeginEndBlock::new(
        "tabularx",
        OptArgs::default(),
        Args::new(vec![
            width.into_tex_element(),
            column_definitions.into_tex_element(),
        ]),
        children,
    )
}

/// Creates a new `textbf` element.
#[inline]
pub fn textbf<T: IntoTexElement>(inner: T) -> MacroCall {
    MacroCall::new_inline("textbf", OptArgs::default(), Args::single(inner))
}

/// Creates a `usepackage` declaration.
#[inline]
pub fn usepackage<T: IntoTexElement>(
    opt_args: Vec<Box<dyn TexElement>>,
    package_name: T,
) -> MacroCall {
    MacroCall::new("usepackage", OptArgs(opt_args), Args::single(package_name))
}

/// Creates a `vspace` element.
#[inline]
pub fn vspace<T: IntoTexElement>(space: T) -> MacroCall {
    MacroCall::new_inline("vspace", OptArgs::default(), Args::single(space))
}
