//! LaTeX-rendering
//!
//! A thin wrapper around external tools like `latexmk`. See `TexRender` for details.

pub mod tex_escape;
pub mod tpl;

use std::{
    ffi::{OsStr, OsString},
    fs, io, path, process,
};
use thiserror::Error;

/// LaTeX-rendering command.
///
/// Creating a new rendering command usually starts by supplying a LaTeX-document, either via
/// `from_bytes` or `from_file`. Other options can be set through the various builder methods.
///
/// Once satisfied, the `render` method will perform the call to the external TeX-engine and return
/// a rendered PDF as raw bytes.
///
/// # TEXINPUTS
///
/// The search path for classes, includes or other files used in TeX can be extended using the
/// `TEXINPUTS` environment variable; `TexRender` sets this variable when rendering. See
/// `add_texinput` for details.
///
/// # Assets
///
/// Instead of adding a folder to `TEXINPUTS`, any sort of external file can be added as an asset.
/// Assets are stored in a temporary folder that lives as long as the `TexRender` instance, the
/// folder will automatically be added to `TEXINPUTS` when rendering. See the `add_asset_*`
/// functions for details.
#[derive(Debug)]
pub struct TexRender {
    /// Content to render.
    source: Vec<u8>,
    /// A number of folders to add to `TEXINPUTS`.
    texinputs: Vec<path::PathBuf>,
    /// Path to latexmk.
    latex_mk_path: path::PathBuf,
    /// Whether or not to use XeLaTeX.
    use_xelatex: bool,
    /// Whether or not to allow shell escaping.
    allow_shell_escape: bool,
    /// Temporary directory holding assets to be included.
    assets_dir: Option<tempdir::TempDir>,
}

/// Error occuring during rendering.
#[derive(Debug, Error)]
pub enum RenderingError {
    /// Temporary directry could not be created.
    #[error("could not create temporary directory: {0}")]
    TempdirCreation(io::Error),
    /// Writing the input file failed.
    #[error("could not write input file: {0}")]
    WriteInputFile(io::Error),
    /// Reading the resulting output file failed.
    #[error("could not read output file: {0}")]
    ReadOutputFile(io::Error),
    /// Could not run LaTeX rendering command.
    #[error("could not run latexmk: {0}")]
    RunError(io::Error),
    /// latexmk failed.
    #[error("LaTeX failure: {stdout:?} {stderr:?}")]
    LatexError {
        /// Process exit code.
        status: Option<i32>,
        /// Content of stdout.
        stdout: Vec<u8>,
        /// Content of stderr.
        stderr: Vec<u8>,
    },
}

impl TexRender {
    /// Create a new tex render configuration using raw input bytes as the source file.
    pub fn from_bytes(source: Vec<u8>) -> TexRender {
        TexRender {
            source,
            texinputs: Vec::new(),
            latex_mk_path: "latexmk".into(),
            use_xelatex: true,
            allow_shell_escape: false,
            assets_dir: None,
        }
    }

    /// Create a new tex render configuration from an input latex file.
    pub fn from_file<P: AsRef<path::Path>>(source: P) -> io::Result<TexRender> {
        Ok(Self::from_bytes(fs::read(source)?))
    }

    /// Adds an asset to the texrender.
    pub fn add_asset_from_bytes<S: AsRef<OsStr>>(
        &mut self,
        filename: S,
        bytes: &[u8],
    ) -> io::Result<()> {
        // Initialize assets dir, if not present.
        let assets_path = match self.assets_dir {
            Some(ref assets_dir) => assets_dir.path(),
            None => {
                let assets_dir = tempdir::TempDir::new("texrender-assets")?;
                self.texinputs.push(assets_dir.path().to_owned());
                self.assets_dir = Some(assets_dir);
                &self.texinputs[self.texinputs.len() - 1]
            }
        };

        let output_fn = assets_path.join(filename.as_ref());
        fs::create_dir_all(output_fn.parent().expect("filename has no parent?"))?;

        fs::write(output_fn, bytes)
    }

    /// Adds an assets to the texrender from a file.
    ///
    /// # Panics
    ///
    /// Panics if the passed-in path has no proper filename.
    pub fn add_asset_from_file<P: AsRef<path::Path>>(&mut self, path: P) -> io::Result<()> {
        let source = path.as_ref();
        let filename = source.file_name().expect("file has no filename");

        let buf = fs::read(source)?;
        self.add_asset_from_bytes(filename, &buf)
    }

    /// Adds a path to list of texinputs.
    pub fn add_texinput<P: Into<path::PathBuf>>(&mut self, input_path: P) -> &mut Self {
        self.texinputs.push(input_path.into());
        self
    }

    /// Sets the path of `latexmk`.
    ///
    /// If not set, will look for `latexmk` on the current `PATH`.
    pub fn latex_mk_path<P: Into<path::PathBuf>>(&mut self, latex_mk_path: P) -> &mut Self {
        self.latex_mk_path = latex_mk_path.into();
        self
    }

    /// Renders the given source as PDF.
    pub fn render(&self) -> Result<Vec<u8>, RenderingError> {
        let tmp = tempdir::TempDir::new("texrender").map_err(RenderingError::TempdirCreation)?;
        let input_file = tmp.path().join("input.tex");
        let output_file = tmp.path().join("input.pdf");

        let mut texinputs = OsString::new();
        for input in &self.texinputs {
            texinputs.push(":");
            texinputs.push(input.as_os_str());
        }

        fs::write(&input_file, &self.source).map_err(RenderingError::WriteInputFile)?;

        let mut cmd = process::Command::new(&self.latex_mk_path);
        cmd.args(&[
            "-interaction=batchmode",
            "-halt-on-error",
            "-file-line-error",
            "-pdf",
        ]);

        if self.use_xelatex {
            cmd.arg("-xelatex");
        }

        if !self.allow_shell_escape {
            cmd.arg("-no-shell-escape");
        }

        cmd.arg(&input_file);

        cmd.env("TEXINPUTS", texinputs);
        cmd.current_dir(tmp.path());

        let output = cmd.output().map_err(RenderingError::RunError)?;

        if !output.status.success() {
            // latexmk failed,
            return Err(RenderingError::LatexError {
                status: output.status.code(),
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }

        fs::read(output_file).map_err(RenderingError::ReadOutputFile)
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderingError, TexRender};

    #[test]
    fn render_example_tex() {
        let doc = r"
        \documentclass{article}
        \begin{document}
        hello, world.
        \end{document}
        ";

        let tex = TexRender::from_bytes(doc.into());
        let _pdf = tex.render().unwrap();
    }

    #[test]
    fn broken_tex_gives_correct_error() {
        let doc = r"
        \documentSOBROKENclass{article}
        ";

        let tex = TexRender::from_bytes(doc.into());

        match tex.render() {
            Err(RenderingError::LatexError { .. }) => (),
            other => panic!("expected latex error, got {:?}", other),
        }
    }
}
