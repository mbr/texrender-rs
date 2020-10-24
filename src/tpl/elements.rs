use super::{Args, BeginEndBlock, Group, MacroCall, OptArgs, RawTex, TexElement, Text};

#[derive(Copy, Clone, Debug)]
pub struct Nothing;

impl Iterator for Nothing {
    type Item = String;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub const N: Nothing = Nothing;

pub fn t<S: Into<String>>(s: S) -> Text {
    Text::new(s)
}

pub fn doc(children: Vec<Box<dyn TexElement>>) -> Group {
    Group(children)
}

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

pub fn document(children: Vec<Box<dyn TexElement>>) -> BeginEndBlock {
    BeginEndBlock::new("document", OptArgs::default(), Args::default(), children)
}

pub fn section<S: Into<String>>(title: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("section"),
        opt_args: OptArgs::default(),
        args: Args(elems![t(title)]),
        newline: true,
    }
}

pub fn subsection<S: Into<String>>(title: S) -> MacroCall {
    MacroCall {
        ident: RawTex::new("subsection"),
        opt_args: OptArgs::default(),
        args: Args(elems![t(title)]),
        newline: true,
    }
}
