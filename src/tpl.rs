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
//! contents.push(Box::new(Text::new("This is Tex.")));
//! let document = BeginEndBlock::new("document", Default::default(), Default::default(), contents);
//! let tex = Group::new(vec![Box::new(doctype) as Box<dyn TexElement>, Box::new(document)]);
//! let output = tex.render().expect("rendering failed");
//! assert_eq!(output,
//!            "\\documentclass[12pt]{article}\n\
//!             \\begin{document}\n\
//!             \\section{Hello, world}\n\
//!             This is Tex.\n\
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
//!         t("This is Tex.")
//!     ))
//! ));
//!
//! let output = tex.render().expect("rendering failed");
//!
//! assert_eq!(output,
//!            "\\documentclass{article}\n\
//!             \\begin{document}\n\
//!             \\section{Hello, world}\n\
//!             This is Tex.\n\
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

pub trait TexElement: Debug {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()>;

    fn render(&self) -> Result<String, string::FromUtf8Error> {
        let mut buffer: Vec<u8> = Vec::new();
        self.write_tex(&mut buffer)
            .expect("should always be able to write to in-memory buffer");
        String::from_utf8(buffer)
    }
}

#[derive(Debug)]
pub struct RawTex(Vec<u8>);

impl RawTex {
    pub fn new<S: Into<String>>(raw: S) -> Self {
        RawTex(raw.into().into_bytes())
    }
}

impl TexElement for RawTex {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self.0.as_slice())
    }
}

#[derive(Debug)]
pub struct Text(String);

impl Text {
    pub fn new<S: Into<String>>(raw: S) -> Self {
        Text(raw.into())
    }
}

impl TexElement for Text {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        // TODO: Escape.
        writer.write_all(self.0.as_bytes());
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct OptArgs(Vec<Box<dyn TexElement>>);

impl OptArgs {
    pub fn new<S: AsRef<str>, I: IntoIterator<Item = S>>(args: I) -> Self {
        Self::new_from_elements(
            args.into_iter()
                .map(|s| Box::new(Text::new(s.as_ref())) as Box<dyn TexElement>)
                .collect(),
        )
    }

    fn new_from_elements(elements: Vec<Box<dyn TexElement>>) -> Self {
        OptArgs(elements)
    }
}

fn write_list<'a, I>(writer: &mut dyn Write, separator: &str, iter: I) -> io::Result<()>
where
    I: Iterator<Item = &'a Box<dyn TexElement>> + 'a,
{
    for (idx, arg) in iter.enumerate() {
        if idx != 0 {
            writer.write_all(b",")?;
        }
        arg.write_tex(writer)?;
    }

    Ok(())
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

#[derive(Debug, Default)]
pub struct Args(Vec<Box<dyn TexElement>>);

impl Args {
    pub fn new<S: AsRef<str>, I: IntoIterator<Item = S>>(args: I) -> Self {
        Self::new_from_elements(
            args.into_iter()
                .map(|s| Box::new(Text::new(s.as_ref())) as Box<dyn TexElement>)
                .collect(),
        )
    }

    fn new_from_elements(elements: Vec<Box<dyn TexElement>>) -> Self {
        Args(elements)
    }
}

impl TexElement for Args {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        if !self.0.is_empty() {
            writer.write_all(b"{")?;
            write_list(writer, ",", self.0.iter())?;
            writer.write_all(b"}")?;
        }

        Ok(())
    }
}

/// A TeX-macro invocation.
///
/// Typically
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
    pub fn new(ident: RawTex, opt_args: OptArgs, args: Args) -> Self {
        MacroCall {
            ident,
            opt_args,
            args,
            newline: true,
        }
    }

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
///
/// The supports optional and mandatory arguments, as well as child elements.
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

#[derive(Debug)]
pub struct AnonymousBlock(Vec<Box<dyn TexElement>>);

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

#[derive(Debug)]
pub struct Group(Vec<Box<dyn TexElement>>);

impl Group {
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
