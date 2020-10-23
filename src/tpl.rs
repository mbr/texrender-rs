use std::fmt::Debug;
use std::io;
use std::io::Write;

macro_rules! elems {
  ($($elem:expr),*) => { {
    let mut collected = Vec::new();
    $(
      collected.push(Box::new($elem) as Box<dyn TexElement>)
      ;)*
    collected
  } };
}

macro_rules! raw {
    ($x:expr) => {
        RawTex::new($x)
    };
}

macro_rules! t {
    ($x:expr) => {
        Text::new($x)
    };
}

trait TexElement: Debug {
    fn write_tex(&self, writer: &mut dyn Write) -> io::Result<()>;
    fn to_string(&self) -> String {
        let mut buffer: Vec<u8> = Vec::new();
        self.write_tex(&mut buffer)
            .expect("should always be able to write to buffer");
        String::from_utf8(buffer)
            .to_owned()
            .expect("generated invalid utf8")
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

// HIGHER LAYER BELOW

fn root(children: Vec<Box<dyn TexElement>>) -> Group {
    Group(children)
}

fn documentclass<S: Into<String>>(opt_args: Vec<Box<dyn TexElement>>, doc_class: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("documentclass"),
        opt_args: OptArgs(opt_args),
        args: Args(vec![Box::new(RawTex::new(doc_class)) as Box<dyn TexElement>]),
        newline: true,
    }
}

fn usepackage<S: Into<String>>(opt_args: Vec<Box<dyn TexElement>>, package_name: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("usepackage"),
        opt_args: OptArgs(opt_args),
        args: Args(vec![
            Box::new(RawTex::new(package_name)) as Box<dyn TexElement>
        ]),
        newline: true,
    }
}

fn document(children: Vec<Box<dyn TexElement>>) -> BeginEndBlock {
    BeginEndBlock::new(OptArgs::default(), Args::default(), "document", children)
}

fn section<S: Into<String>>(title: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("section"),
        opt_args: OptArgs::default(),
        args: Args(elems![t!(title.into())]),
        newline: true,
    }
}

fn subsection<S: Into<String>>(title: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("subsection"),
        opt_args: OptArgs::default(),
        args: Args(elems![t!(title.into())]),
        newline: true,
    }
}

#[cfg(test)]
mod tests {
    use super::TexElement;
    use super::*;

    #[test]
    fn simple_example() {
        let doc = document(elems![
            section("Notes"),
            t!("I wrote some interesting stuff today\n"),
            subsection("Subnotes")
        ]);

        let tex = root(elems![
            documentclass(elems![t!("12pt")], "article"),
            usepackage(vec![], "lingmacros"),
            usepackage(vec![], "tree-dvips"),
            doc
        ]);

        eprintln!("{}", tex.to_string());
    }
}
