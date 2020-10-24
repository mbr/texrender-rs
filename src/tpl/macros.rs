//! Macros used for tex templating.

/// Box and type erase a number of tex elements.
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
