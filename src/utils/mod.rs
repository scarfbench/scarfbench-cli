use std::io::{self, BufRead, Read};

use kdam::{BarExt, Column, RichProgress, Spinner, tqdm};
use owo_colors::OwoColorize;
use std::io::{Error, ErrorKind, Result};

pub struct ProgressReader<R: Read> {
    /// Anything that has a read() method—a file, a network stream, anything...
    inner: R,
    /// kdam's progress bar
    pb: RichProgress,
    /// Total Size
    total: Option<usize>,
}
impl<R: Read> ProgressReader<R> {
    pub fn new(inner: R, pb: RichProgress, total: Option<usize>) -> Self {
        Self { inner, pb, total }
    }
}
impl<R: Read> Read for ProgressReader<R> {
    /// When someone calls this wrapper's read, do the following:
    /// 1. Forward that to the inner reader
    /// 2. Send the count of bytes read to the progress bar from kdam
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        if bytes_read > 0 {
            self.pb
                .update(bytes_read)
                .map_err(|e| Error::new(ErrorKind::Other, e))?;
        }
        Result::Ok(bytes_read)
    }
}
impl<R: BufRead> BufRead for ProgressReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        // Important: count bytes that are consumed from the buffer as "read"
        if amt > 0 {
            // `kdam` update returns a Result, but BufRead::consume can't.
            // Best effort: ignore error or store it somewhere.
            let _ = self.pb.update(amt);
        }
        self.inner.consume(amt);
    }
}
impl<R: Read> Drop for ProgressReader<R> {
    fn drop(&mut self) {
        if let Some(total) = self.total {
            let _ = self.pb.update_to(total);
        }
    }
}
pub trait ProgressBar {
    fn progress(
        self,
        lable: impl Into<String>,
        unit: impl Into<String>,
    ) -> RichProgress;
}
impl ProgressBar for usize {
    fn progress(
        self,
        lable: impl Into<String>,
        unit: impl Into<String>,
    ) -> RichProgress {
        RichProgress::new(
            tqdm!(
                total = self,
                unit_scale = true,
                unit_divisor = 1024,
                unit = unit.into()
            ),
            vec![
                Column::Spinner(Spinner::new(
                    &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
                    80.0,
                    1.0,
                )),
                Column::Text(
                    format!("[bold blue] {} :: ", lable.into()).to_owned(),
                ),
                Column::Animation,
                Column::Percentage(1),
                Column::Text("•".to_owned()),
                Column::CountTotal,
                Column::Text("•".to_owned()),
                Column::Rate,
                Column::Text("•".to_owned()),
                Column::RemainingTime,
            ],
        )
    }
}
pub(crate) fn logo() -> String {
    format!(
        "\x1b[1m\x1b[31m{}\x1b[0m",
        r#"
 ███████╗ ██████╗ █████╗ ██████╗ ███████╗██████╗ ███████╗███╗   ██╗ ██████╗██╗  ██╗
 ██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔════╝████╗  ██║██╔════╝██║  ██║
 ███████╗██║     ███████║██████╔╝█████╗  ██████╔╝█████╗  ██╔██╗ ██║██║     ███████║
 ╚════██║██║     ██╔══██║██╔══██╗██╔══╝  ██╔══██╗██╔══╝  ██║╚██╗██║██║     ██╔══██║
 ███████║╚██████╗██║  ██║██║  ██║██║     ██████╔╝███████╗██║ ╚████║╚██████╗██║  ██║
 ╚══════╝ ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝╚═╝  ╚═╝"#
            .bold()
            .red()
    )
}
