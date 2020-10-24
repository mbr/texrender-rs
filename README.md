# texrender

A small crate to run `latexmk` from Rust, similar to [latex](https://pypi.org/project/latex/), escape LaTeX code and generate LaTeX documents programmatically.

## Example: Rendering latex

```rust
let doc = r"
      \documentclass{article}
      \begin{document}
      hello, world.
      \end{document}
      ";

let tex = TexRender::from_bytes(doc.into());
let _pdf = tex.render().expect("latex rendering failed");
```

## Example: Generating latex code

```rust
use texrender::elems;
use texrender::tpl::TexElement;
use texrender::tpl::elements::{N, doc, document, documentclass, section, t};

let tex = doc(elems!(
    documentclass(N, "article"),
    document(elems!(
        section("Hello, world"),
        t("This is fun & easy.")
    ))
));

let output = tex.render().expect("rendering failed");

assert_eq!(output,
           "\\documentclass{article}\n\
            \\begin{document}\n\
            \\section{Hello, world}\n\
            This is fun \\& easy.\n\
            \\end{document}\n");
```
