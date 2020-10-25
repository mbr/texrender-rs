//! Predefined LaTeX elements.
//!
//! Contains a number of elements that can be used to generate LaTeX code. See the `tpl` module
//! documentation for an example and comparison.

use super::{Args, BeginEndBlock, Group, MacroCall, OptArgs, RawTex, TexElement, Text};

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

/// Creates a new `document` begin/end block.
#[inline]
pub fn document(children: Vec<Box<dyn TexElement>>) -> BeginEndBlock {
    BeginEndBlock::new("document", OptArgs::default(), Args::default(), children)
}

/// Creates an anonymous group.
#[inline]
pub fn group(children: Vec<Box<dyn TexElement>>) -> Group {
    Group(children)
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
