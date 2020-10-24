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
