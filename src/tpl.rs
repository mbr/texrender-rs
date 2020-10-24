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
//!# use tpl::{TexElement, MacroCall};
//!#
//! let doctype = MacroCall::new("documentclass",
//! //Default::default(), Default::default());
//!                              OptArgs::new(["12pt"])
//!                              Args::new(["article"]));
//! let output = doctype.to_string().expect("rendering failed");
//! ```

use std::fmt::Debug;
use std::io::Write;
use std::{io, string};

trait TexElement: Debug {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()>;

    fn to_string(&self) -> Result<String, string::FromUtf8Error> {
        let mut buffer: Vec<u8> = Vec::new();
        self.write_tex(&mut buffer)
            .expect("should always be able to write to in-memory buffer");
        String::from_utf8(buffer)
    }
}

#[derive(Debug)]
struct RawTex(Vec<u8>);

impl RawTex {
    fn new<S: Into<String>>(raw: S) -> Self {
        RawTex(raw.into().into_bytes())
    }
}

impl TexElement for RawTex {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self.0.as_slice())
    }
}

#[derive(Debug)]
struct Text(String);

impl Text {
    fn new<S: Into<String>>(raw: S) -> Self {
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
struct OptArgs(Vec<Box<dyn TexElement>>);

impl OptArgs {
    fn new<S: Into<String>, I: IntoIterator<Item = S>>(args: I) -> Self {
        Self::new_from_elements(
            args.into_iter()
                .map(|s| Box::new(Text::new(s)) as Box<dyn TexElement>)
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
struct Args(Vec<Box<dyn TexElement>>);

impl Args {
    fn new<S: Into<String>, I: IntoIterator<Item = S>>(args: I) -> Self {
        Self::new_from_elements(
            args.into_iter()
                .map(|s| Box::new(Text::new(s)) as Box<dyn TexElement>)
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
    fn new(ident: RawTex, opt_args: OptArgs, args: Args) -> Self {
        MacroCall {
            ident,
            opt_args,
            args,
            newline: true,
        }
    }

    fn new_inline(ident: RawTex, opt_args: OptArgs, args: Args) -> Self {
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
    fn new<S: Into<String>>(
        opt_args: OptArgs,
        args: Args,
        ident: S,
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

impl TexElement for Group {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()> {
        for child in &self.0 {
            child.write_tex(writer)?;
        }
        Ok(())
    }
}
