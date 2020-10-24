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

/// Creates a new text element.
pub fn t<S: Into<String>>(s: S) -> Text {
    Text::new(s)
}

/// Creates a new top-level document.
pub fn doc(children: Vec<Box<dyn TexElement>>) -> Group {
    Group(children)
}

/// Creates a `documentclass` declaration.
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

/// Creates a `usepackage` declaration.
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

/// Creates a new `document` begin/end block.
pub fn document(children: Vec<Box<dyn TexElement>>) -> BeginEndBlock {
    BeginEndBlock::new("document", OptArgs::default(), Args::default(), children)
}

/// Creates a new `section` header.
pub fn section<S: Into<String>>(title: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("section"),
        opt_args: OptArgs::default(),
        args: Args(elems![t(title)]),
        newline: true,
    }
}

/// Creates a new `subsection` header.
pub fn subsection<S: Into<String>>(title: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("subsection"),
        opt_args: OptArgs::default(),
        args: Args(elems![t(title)]),
        newline: true,
    }
}
