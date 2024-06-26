use eyre::{Result, WrapErr};
use std::io::Write;

pub mod coqtop;

// Re-export `roq_derive` when `derive` feature is enabled.
#[cfg(feature = "derive")]
pub use roq_derive::*;

#[macro_export]
macro_rules! try_prove {
    ($($tag:ident $t:tt),* $(,)?) => {
        {
            let mut batch = String::new();
            $(
                batch.push_str(match stringify!($tag) {
                    "inline" => "(** ** inline *)\n".to_string(),
                    "file" => format!(
                        "(** ** file: {} *)\n",
                        stringify!($t)
                    ),
                    "function" => format!(
                        "(** ** function: {} *)\n",
                        stringify!($t)
                    ),
                    _ => "(** ** chunk *)\n".to_string(),
                }.as_str());
                batch.push_str($crate::_part_to_str!($tag $t));
                batch.push_str("\n\n\n");
            )*

            eprintln!("```coq");
            eprintln!("{}", batch);
            eprintln!("```");

            $crate::_try_prove(&batch)
        }
    }
}

#[macro_export]
macro_rules! prove {
    ($($tag:ident $t:tt),* $(,)?) => {
        $crate::try_prove!($($tag $t),*).unwrap()
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! _part_to_str {
    (inline $s:literal) => {
        $s
    };
    (file $file:literal) => {
        include_str!($file)
    };
    (function $f:ident) => {
        $f::roq::vernacular().to_string().as_str()
    };
}

/// Stick the source into a temporary file, and run it through Coq.
#[doc(hidden)]
pub fn _try_prove(source: &str) -> Result<String> {
    // Make a tempfile for each.
    let mut file = tempfile::Builder::new()
        .prefix("roq_")
        .suffix(".v")
        .rand_bytes(8)
        .tempfile()
        .wrap_err("Failed to create temp file for Coq vernacular")?;
    write!(file, "{}", source).wrap_err("Failed to write Coq vernacular to tempfile")?;
    coqtop::Coqtop::new().run_batch(&[file.as_ref()])
}
