use std::{
    fs::File,
    io::{self, stdout, BufRead, BufReader},
};

fn do_file(r: impl std::io::Read, w: impl std::io::Write) -> std::io::Result<()> {
    let bufr = BufReader::new(r);
    for line in bufr.lines() {
        println!("{}", line?);
    }

    todo!()
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
    type Item = Result<Vec<String>, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        println!("next");
        todo!()
    }
}

#[cfg(test)]
mod tests {
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
    fn can_read_log_line_correctly() {
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

fn main() {
    let out = stdout();
    let args = std::env::args().skip(1);
    // Open files once to count the maximum number of columns; This is used to pad column count.
    let max_columns: std::io::Result<usize> =
        std::env::args()
            .skip(1)
            .map(File::open)
            .try_fold(0, |prev, file| {
                let n = count_max_columns(file?)?;
                Ok(if prev > n { prev } else { n })
            });

    println!("max cols: {:?}", max_columns);

    // for name in args {
    //     if let Err(error) = File::open(&name).and_then(|mut handle| do_file(&mut handle, &out)) {
    //         eprintln!("could not process {}: {}", name, error);
    //         exit(error.raw_os_error().unwrap_or(-1));
    //     }
    // }
    // Open them again to write out the CSV in the correct format with the correct number of columns.

    println!("Hello, world!");
}
