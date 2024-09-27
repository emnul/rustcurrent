/// Wrote this struct to get a better feeling for
/// Using the newtype pattern https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types
/// in order to modify the behavior of the inner type
///
/// Some brainstorm implementations
struct AddNewlineWriter<W: Write>(W);

impl<W: Write> AddNewlineWriter<W> {
    fn add_newline(&mut self) -> io::Result<()> {
        self.0.write_all(b"\n")
    }
}

impl<W: Write> Write for AddNewlineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(b"\n")
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        // println!();
        // println!();
        if buf.is_empty() {
            // println!("empty buf");
            self.0.write_all(b"\n")
        } else {
            self.0.write_all(buf)
        }
    }
}
