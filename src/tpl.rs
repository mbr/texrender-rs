//! TeX templating
//!
//! The `tpl` module contains a way of constructing a TeX-document programmatically. It ensures
//! documents are well-formed syntactically, but not semantically (e.g. it is possible to express
//! documents that contain multiple `\documentclass` macro calls inside the document but not a
//! `\begin{foo}` without a matching `\end`).
//!
//! As a result of this deliberate limitation, the API is fairly simple. The core module offers the
//! entire abstraction through the `TexElement` trait, while the `elements` module contains
//! syntactic sugar for building documents quickly.
//!
//! ## "Hello, world" using `TexElement` directly.
//!
//! ```rust
//! use texrender::tpl::{Args, BeginEndBlock, Group, MacroCall, OptArgs, RawTex, TexElement, Text};
//!
//! let doctype = MacroCall::new(RawTex::new("documentclass"),
//!                              OptArgs::new(&["12pt"]),
//!                              Args::new(&["article"]));
//! let mut contents: Vec<Box<dyn TexElement>> = Vec::new();
//! contents.push(Box::new(MacroCall::new(RawTex::new("section"),
//!                        Default::default(),
//!                        Args::new(&["Hello, world"]))));
//! contents.push(Box::new(Text::new("This is fun & easy.")));
//! let document = BeginEndBlock::new("document", Default::default(), Default::default(), contents);
//! let tex = Group::new(vec![Box::new(doctype) as Box<dyn TexElement>, Box::new(document)]);
//! let output = tex.render().expect("rendering failed");
//! assert_eq!(output,
//!            "\\documentclass[12pt]{article}\n\
//!             \\begin{document}\n\
//!             \\section{Hello, world}\n\
//!             This is fun \\& easy.\n\
//!             \\end{document}\n");
//! ```
//!
//! While this form uses no macros, it is rather inconvenient to write. Luckily there is an
//! alternative:
//!
//! ## "Hello, world" using elements and macros.
//!
//! ```rust
//! use texrender::elems;
//! use texrender::tpl::TexElement;
//! use texrender::tpl::elements::{N, doc, document, documentclass, section, t};
//!
//! let tex = doc(elems!(
//!     documentclass(N, "article"),
//!     document(elems!(
//!         section("Hello, world"),
//!         t("This is fun & easy.")
//!     ))
//! ));
//!
//! let output = tex.render().expect("rendering failed");
//!
//! assert_eq!(output,
//!            "\\documentclass{article}\n\
//!             \\begin{document}\n\
//!             \\section{Hello, world}\n\
//!             This is fun \\& easy.\n\
//!             \\end{document}\n");
//! ```
//!
//! Element functions like `section` above typically cover most use cases, while not preventing the
//! u ser to drop back to the raw functions above. The `elems` macro conveniently boxes and
//! type-erases children, while `N` can be used for "no arguments" for both args and optargs.

#[macro_use]
pub mod macros;

pub mod elements;

use std::fmt::Debug;
use std::io::Write;
use std::{io, string};

/// Renderable Tex element.
pub trait TexElement: Debug {
    /// Writes a rendering of the element to the given writer.
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()>;

    /// Renders the element into a string.
    ///
    /// May return an error if a non-utf8 element has been given.
    fn render(&self) -> Result<String, string::FromUtf8Error> {
        let mut buffer: Vec<u8> = Vec::new();
        self.write_tex(&mut buffer)
            .expect("should always be able to write to in-memory buffer");
        String::from_utf8(buffer)
    }
}

/// Conversion trait for various types.
///
/// Used for primitive conversions of various types directly into tex elements. Implementations
/// include:
///
/// * `str` and `String` are converted to escaped `Text` elements.
/// * Any number (`u8`, ...) is converted to escaped `Text` using display.
pub trait IntoTexElement {
    /// Converts the given element into a `TexElement`.
    fn into_tex_element(&self) -> Box<dyn TexElement>;
}

impl<'a> IntoTexElement for &'a str {
    fn into_tex_element(&self) -> Box<dyn TexElement> {
        Box::new(Text::new(self.to_owned()))
    }
}

impl IntoTexElement for String {
    fn into_tex_element(&self) -> Box<dyn TexElement> {
        Box::new(Text::new(self))
    }
}

macro_rules! using_display {
    ($ty:ty) => {
        impl IntoTexElement for $ty {
            fn into_tex_element(&self) -> Box<dyn TexElement> {
                Box::new(Text::new(format!("{}", self)))
            }
        }
    };
}

using_display!(u8);
using_display!(u16);
using_display!(u32);
using_display!(u64);
using_display!(u128);
using_display!(i8);
using_display!(i16);
using_display!(i32);
using_display!(i64);
using_display!(i128);
using_display!(f32);
using_display!(f64);

/// Writes a list of tex elements to a stream with a separator.
pub fn write_list<'a, I>(writer: &mut dyn Write, separator: &str, iter: I) -> io::Result<()>
where
    I: Iterator<Item = &'a Box<dyn TexElement>> + 'a,
{
    for (idx, arg) in iter.enumerate() {
        if idx != 0 {
            writer.write_all(separator.as_bytes())?;
        }
        arg.write_tex(writer)?;
    }

    Ok(())
}

/// A raw, unescaped piece of tex code.
///
/// Tex is not guaranteed to be UTF-8 encoded, thus `RawTex` internally keeps bytes. The value will
/// be inserted into the document without any escaping. The value is unchecked, thus it is possible
/// to create syntactically incorrect invalid documents using this element.
#[derive(Debug)]
pub struct RawTex(Vec<u8>);

impl RawTex {
    /// Crates a new raw tex element from a string.
    #[inline]
    pub fn new<S: Into<String>>(raw: S) -> Self {
        RawTex(raw.into().into_bytes())
    }

    /// Crates a new raw tex element from a bytes.
    #[inline]
    pub fn new_bytes(raw: Vec<u8>) -> Self {
        RawTex(raw)
    }
}

impl TexElement for RawTex {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self.0.as_slice())
    }
}

/// A text string.
///
/// Text strings will be escaped before insertion.
#[derive(Debug)]
pub struct Text(String);

impl Text {
    /// Creates a new text string.
    #[inline]
    pub fn new<S: Into<String>>(raw: S) -> Self {
        Text(raw.into())
    }
}

impl TexElement for Text {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        crate::tex_escape::write_escaped(writer, &self.0)
    }
}

/// A set of optional arguments.
///
/// Optional arguments in LaTeX are typically denoted using square brackets and comma-separated.
#[derive(Debug, Default)]
pub struct OptArgs(Vec<Box<dyn TexElement>>);

impl OptArgs {
    /// Creates a new set of optional arguments.
    ///
    /// This is a higher level convenience functions that accepts iterables of to-be-escaped
    /// strings, see `new_from_elements` for a lower-level one.
    #[inline]
    pub fn new<S: AsRef<str>, I: IntoIterator<Item = S>>(args: I) -> Self {
        Self::new_from_elements(
            args.into_iter()
                .map(|s| Box::new(Text::new(s.as_ref())) as Box<dyn TexElement>)
                .collect(),
        )
    }

    /// Crates a new set of optional arguments.
    #[inline]
    pub fn new_from_elements(elements: Vec<Box<dyn TexElement>>) -> Self {
        OptArgs(elements)
    }
}

impl TexElement for OptArgs {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        if !self.0.is_empty() {
            writer.write_all(b"[")?;
            write_list(writer, ",", self.0.iter())?;
            writer.write_all(b"]")?;
        }

        Ok(())
    }
}

/// A set of arguments.
///
/// Each argument is enclosed by curly braces when rendered, otherwise arguments are just
/// concatenated.
#[derive(Debug, Default)]
pub struct Args(Vec<Box<dyn TexElement>>);

impl Args {
    /// Creates a new set of arguments.
    ///
    /// This is a higher level convenience functions that accepts iterables of to-be-escaped
    /// strings, see `new_from_elements` for a lower-level one.
    #[inline]
    pub fn new<S: AsRef<str>, I: IntoIterator<Item = S>>(args: I) -> Self {
        Self::new_from_elements(
            args.into_iter()
                .map(|s| Box::new(Text::new(s.as_ref())) as Box<dyn TexElement>)
                .collect(),
        )
    }

    /// Crates a new set of arguments.
    #[inline]
    pub fn new_from_elements(elements: Vec<Box<dyn TexElement>>) -> Self {
        Args(elements)
    }
}

impl TexElement for Args {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        if !self.0.is_empty() {
            writer.write_all(b"{")?;
            write_list(writer, "}{", self.0.iter())?;
            writer.write_all(b"}")?;
        }

        Ok(())
    }
}

/// A TeX-macro invocation.
///
/// This is the typical `\macroname[opt1]{arg1}{arg2}` call that is common in latex documents.
#[derive(Debug)]
pub struct MacroCall {
    /// Name of the instruction.
    ident: RawTex,
    /// Optional arguments.
    opt_args: OptArgs,
    /// Mandatory arguments.
    args: Args,
    /// Whether or not to append a newline afterwards.
    newline: bool,
}

impl MacroCall {
    /// Creates a new macro call.
    ///
    /// The resulting call will end with a newline when output.
    pub fn new(ident: RawTex, opt_args: OptArgs, args: Args) -> Self {
        MacroCall {
            ident,
            opt_args,
            args,
            newline: true,
        }
    }

    /// Creates a new inline macro call.
    ///
    /// Does not end with a newline.
    pub fn new_inline(ident: RawTex, opt_args: OptArgs, args: Args) -> Self {
        MacroCall {
            ident,
            opt_args,
            args,
            newline: false,
        }
    }
}

impl TexElement for MacroCall {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(br"\")?;
        self.ident.write_tex(writer)?;
        self.opt_args.write_tex(writer)?;
        self.args.write_tex(writer)?;
        if self.newline {
            writer.write_all(b"\n")?;
        }
        Ok(())
    }
}

/// A block with a begin and end instruction.
///
/// Begin-end blocks usually start with a `\begin{blockname}` and end with `\end{blockname}`.
#[derive(Debug)]
pub struct BeginEndBlock {
    /// The opening instruction, typically `\begin{blockname}`.
    opening: MacroCall,
    /// Children.
    children: Vec<Box<dyn TexElement>>,
    /// Closing instruction, typically `\end{blockname}`
    closing: MacroCall,
}

impl BeginEndBlock {
    /// Creates a new begin/end block.
    pub fn new<S: Into<String>>(
        ident: S,
        opt_args: OptArgs,
        args: Args,
        children: Vec<Box<dyn TexElement>>,
    ) -> Self {
        let ident = ident.into();

        let mut opening_args =
            vec![Box::new(RawTex(ident.clone().into_bytes())) as Box<dyn TexElement>];
        let closing_args =
            vec![Box::new(RawTex(ident.clone().into_bytes())) as Box<dyn TexElement>];

        opening_args.extend(args.0.into_iter());

        BeginEndBlock {
            opening: MacroCall {
                ident: RawTex::new("begin"),
                opt_args,
                args: Args(opening_args),
                newline: true,
            },
            children,
            closing: MacroCall {
                ident: RawTex::new("end"),
                opt_args: OptArgs::default(),
                args: Args(closing_args),
                newline: true,
            },
        }
    }
}

impl TexElement for BeginEndBlock {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        self.opening.write_tex(writer)?;

        for child in &self.children {
            child.write_tex(writer)?;
        }

        writer.write_all(b"\n")?;
        self.closing.write_tex(writer)?;

        Ok(())
    }
}

/// An anonymous block.
///
/// Anonymous blocks are other elements enclosed in curly braces when output.
#[derive(Debug)]
pub struct AnonymousBlock(Vec<Box<dyn TexElement>>);

impl AnonymousBlock {
    /// Creates a new anonymous block.
    pub fn new(elems: Vec<Box<dyn TexElement>>) -> Self {
        AnonymousBlock(elems)
    }
}

impl TexElement for AnonymousBlock {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(b"{")?;
        for child in &self.0 {
            child.write_tex(writer)?;
        }
        writer.write_all(b"}")?;
        Ok(())
    }
}

/// Grouping of elements.
///
/// Groups multiple elements together; when output they are written in order, without any characters
/// added.
#[derive(Debug)]
pub struct Group(Vec<Box<dyn TexElement>>);

impl Group {
    /// Creates a new group.
    pub fn new(elems: Vec<Box<dyn TexElement>>) -> Self {
        Group(elems)
    }
}

impl TexElement for Group {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        for child in &self.0 {
            child.write_tex(writer)?;
        }
        Ok(())
    }
}

/// Table row.
///
/// Multiple elements joined by ` & ` when rendered.
#[derive(Debug)]
pub struct TableRow(Vec<Box<dyn TexElement>>);

impl TableRow {
    /// Creates a new table row.
    pub fn new(elems: Vec<Box<dyn TexElement>>) -> Self {
        TableRow(elems)
    }
}

impl TexElement for TableRow {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        write_list(writer, " & ", self.0.iter())?;
        writer.write_all(b"\\\\\n")
    }
}
