#[macro_export]
macro_rules! elems {
  ($($elem:expr),*) => { {
    let mut collected = Vec::new();
    $(
      collected.push(Box::new($elem) as Box<dyn $crate::tpl::TexElement>)
      ;)*
      collected
    } };
  }
