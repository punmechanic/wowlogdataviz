use std::{
    fs::File,
    io::{self, stdout, BufRead, BufReader},
    process::exit,
};

/// When returned, indicates that the given line opened more square braces than it closed, or closed more than it opened
///
/// This indicates a log line is malformed or split across multiple lines, which is not valid in WoW combat logs.
#[derive(Debug, PartialEq)]
enum ReadLineError {
    MalformedLine,
    StackOverflow,
}

impl From<ReadLineError> for io::Error {
    fn from(val: ReadLineError) -> Self {
        io::Error::new(
            io::ErrorKind::InvalidData,
            match val {
                ReadLineError::MalformedLine => "malformed line",
                ReadLineError::StackOverflow => "stack overflow",
            },
        )
    }
}

struct LogReader<R> {
    r: R,
}

impl<R> LogReader<R>
where
    R: io::Read,
{
    fn new(r: R) -> Self {
        Self { r }
    }

    fn lines(self) -> Lines<impl Iterator<Item = io::Result<String>>> {
        let buf = BufReader::new(self.r);
        Lines(buf.lines())
    }
}

struct Lines<I>(I)
where
    I: Iterator<Item = io::Result<String>>;

impl<I> Iterator for Lines<I>
where
    I: Iterator<Item = io::Result<String>>,
{
    type Item = io::Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.0.next() {
            match line {
                Ok(line) => LineReader::new(line).next().map(|x| x.unwrap()).map(Ok),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }
}

struct LineReader {
    source: String,
    // It's unlikely Blizzard will ever go over 255 chars
    stack: u8,
    pos: usize,
}

impl LineReader {
    fn new(source: String) -> Self {
        Self {
            source,
            stack: 0,
            pos: 0,
        }
    }

    fn next_cell(&mut self) -> Result<Option<String>, ReadLineError> {
        // We read until the next comma
        let slice = &self.source[self.pos..];
        if slice.is_empty() {
            // Nothing left
            return Ok(None);
        }

        for (idx, char) in slice.char_indices() {
            self.pos += 1;
            if char == '[' {
                if self.stack == u8::MAX {
                    return Err(ReadLineError::StackOverflow);
                }

                self.stack += 1;
                continue;
            }

            if char == ']' {
                if self.stack == 0 {
                    // Tried to pop from the stack when nothing was on there - indicates a malformed result
                    return Err(ReadLineError::MalformedLine);
                }
                self.stack -= 1;

                continue;
            }

            if char == ',' && self.stack == 0 {
                // Nothing on the stack, so we can yield
                return Ok(Some(slice[0..idx].to_string()));
            }
        }

        if self.stack == 0 {
            Ok(Some(slice.to_string()))
        } else {
            // The stack is not empty which means something was not terminated and the line was incomplete
            Err(ReadLineError::MalformedLine)
        }
    }
}

impl Iterator for LineReader {
    type Item = Result<Vec<String>, ReadLineError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = Vec::new();

        loop {
            match self.next_cell() {
                Ok(None) => break,
                Ok(Some(cell)) => line.push(cell),
                Err(e) => return Some(Err(e)),
            }
        }

        if line.is_empty() {
            None
        } else {
            Some(Ok(line))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LineReader;
    use crate::ReadLineError;
    use std::{io::Read, io::Result, slice::Iter};

    pub struct StringReader<'a> {
        iter: Iter<'a, u8>,
    }

    impl<'a> StringReader<'a> {
        /// Wrap a string in a `StringReader`, which implements `std::io::Read`.
        pub fn new(data: &'a str) -> Self {
            Self {
                iter: data.as_bytes().iter(),
            }
        }
    }

    impl<'a> Read for StringReader<'a> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            for i in 0..buf.len() {
                if let Some(x) = self.iter.next() {
                    buf[i] = *x;
                } else {
                    return Ok(i);
                }
            }
            Ok(buf.len())
        }
    }

    #[test]
    fn line_reader_reports_malformed_input_correctly() {
        let doc = "foo,bar,[";
        let mut iter = LineReader::new(doc.into());
        let res = iter.next().unwrap();
        assert_eq!(res, Err(ReadLineError::MalformedLine));
    }

    #[test]
    fn line_reader_can_read_log_line_correctly() {
        let doc = "4/5 18:51:38.090  COMBATANT_INFO,Player-57-0CF2A300,0,1215,1731,17267,10131,0,0,0,1118,1118,1118,250,181,5897,5897,5897,570,3461,1257,1257,1257,2020,258,[(82710,103678,1),(82654,103679,1)]";
        let mut iter = LineReader::new(doc.into());
        let res = iter
            .next()
            .expect("had no line")
            .expect("could not read line");

        let expected = vec![
            "4/5 18:51:38.090  COMBATANT_INFO",
            "Player-57-0CF2A300",
            "0",
            "1215",
            "1731",
            "17267",
            "10131",
            "0",
            "0",
            "0",
            "1118",
            "1118",
            "1118",
            "250",
            "181",
            "5897",
            "5897",
            "5897",
            "570",
            "3461",
            "1257",
            "1257",
            "1257",
            "2020",
            "258",
            "[(82710,103678,1),(82654,103679,1)]",
        ];

        assert_eq!(res, expected);
        assert!(iter.next().is_none());
    }

    #[test]
    fn log_reader_can_read_log_line_correctly() {
        let doc = "4/5 16:34:03.029  COMBAT_LOG_VERSION,20,ADVANCED_LOG_ENABLED,1,BUILD_VERSION,10.0.7,PROJECT_ID,1";
        let r = StringReader::new(doc);
        let r = super::LogReader::new(r);
        let mut iter = r.lines();
        let res = iter
            .next()
            .expect("had no line")
            .expect("could not read line");

        let expected = vec![
            "4/5 16:34:03.029",
            "COMBAT_LOG_VERSION",
            "20",
            "ADVANCED_LOG_ENABLED",
            "1",
            "BUILD_VERSION",
            "10.0.7",
            "PROJECT_ID",
            "1",
        ];

        assert_eq!(res, expected);
        assert!(iter.next().is_none());
    }
}

fn count_max_columns(r: impl std::io::Read) -> io::Result<usize> {
    LogReader::new(BufReader::new(r))
        .lines()
        .try_fold(0, |prev, line| {
            // There's one extra column because the first column will be both timestamp and event.
            // TODO: I expect this could be much faster if we counted the incidences of , in the string rather than splitting it.
            let line = line?;
            let n = line.len() + 1;
            if n > 400 {
                println!("{:?}", line);
            }

            Ok(if prev > n { prev } else { n })
        })
}

fn count_many_max_columns(rs: impl Iterator<Item = String>) -> io::Result<usize> {
    rs.map(File::open).try_fold(0, |prev, file| {
        let n = count_max_columns(file?)?;
        Ok(if prev > n { prev } else { n })
    })
}

fn filenames() -> impl Iterator<Item = String> {
    std::env::args().skip(1)
}

fn pad_columns(r: impl std::io::Read, w: impl std::io::Write, n: usize) -> std::io::Result<()> {
    let bufr = BufReader::new(r);
    let r = LogReader::new(bufr);
    let mut w = PaddedCsvWriter::new(w, n);
    for line in r.lines() {
        w.write(line?)?;
    }

    Ok(())
}

struct PaddedCsvWriter<W>(W, usize)
where
    W: io::Write;

impl<W> PaddedCsvWriter<W>
where
    W: io::Write,
{
    fn new(w: W, count: usize) -> Self {
        Self(w, count)
    }

    fn write(&mut self, cells: Vec<String>) -> io::Result<()> {
        let mut output = cells.join(",");
        output.extend((cells.len()..self.1).map(|_| ","));
        let bytes = output.as_bytes();
        self.0.write_all(bytes)?;
        self.0.write_all(b"\n")?;
        Ok(())
    }
}

fn main() {
    // Open files once to count the maximum number of columns; This is used to pad column count.
    let max_columns = match count_many_max_columns(filenames()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("could not process: {}", e);
            exit(e.raw_os_error().unwrap_or(-1));
        }
    };

    // Now we do it again, but we output lines padded to equal the maximum column length.
    // This lets us use the files in things like Excel.
    let out = stdout();
    for name in filenames() {
        if let Err(error) = File::open(&name).map(|handle| pad_columns(handle, &out, max_columns)) {
            eprintln!("could not process {}: {}", name, error);
            exit(error.raw_os_error().unwrap_or(-1));
        }
    }
}
