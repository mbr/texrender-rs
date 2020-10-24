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
