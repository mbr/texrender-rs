# texrender

A small crate to run `latexmk` from Rust, similar to [latex](https://pypi.org/project/latex/).

Example:

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
