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
    type Item = io::Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().and_then(|r| {
            // TODO: Hacky
            if r.is_err() {
                let res = Err(r.err().unwrap());
                Some(res)
            } else {
                LineReader::new(r.unwrap()).next()
            }
        })
    }
}

struct LineReader {
    source: String,
    stack: Vec<(usize, char)>,
    pos: usize,
}

impl LineReader {
    fn new(source: String) -> Self {
        Self {
            source,
            stack: Vec::new(),
            pos: 0,
        }
    }

    fn next_cell(&mut self) -> Option<String> {
        // We read until the next comma
        let slice = &self.source[self.pos..];
        if slice.is_empty() {
            // Nothing left
            return None;
        }

        for (idx, char) in slice.char_indices() {
            self.pos += 1;
            if char == '[' {
                self.stack.push((idx, char));
                continue;
            }

            if char == ']' {
                if self.stack.pop() == None {
                    // Tried to pop from the stack when nothing was on there - indicates a malformed result
                    todo!();
                }

                continue;
            }

            if char == ',' && self.stack.is_empty() {
                // Nothing on the stack, so we can yield
                return Some(slice[0..idx].to_string());
            }
        }

        if self.stack.is_empty() {
            Some(slice.to_string())
        } else {
            // The stack is not empty which means something was not terminated and the line was incomplete
            todo!("incomplete line");
        }
    }
}

impl Iterator for LineReader {
    type Item = io::Result<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = Vec::new();
        while let Some(cell) = self.next_cell() {
            println!("{}", cell);
            line.push(cell);
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
    fn line_reader_can_read_log_line_correctly() {
        let doc = "4/5 18:51:38.090  COMBATANT_INFO,Player-57-0CF2A300,0,1215,1731,17267,10131,0,0,0,1118,1118,1118,250,181,5897,5897,5897,570,3461,1257,1257,1257,2020,258,[(82710,103678,1),(82654,103679,1),(82657,103680,1),(82559,103682,1),(82562,103685,1),(82646,103785,1),(82650,103791,1),(82653,103795,1),(82656,103798,2),(82557,103803,1),(82662,103805,1),(82663,103806,1),(82664,103807,1),(82665,103808,1),(82666,103809,1),(82667,103811,1),(82668,103812,1),(82669,103813,1),(82672,103818,2),(82673,103819,1),(82675,103821,1),(82679,103826,2),(82680,103827,1),(82683,103832,1),(82684,103833,2),(82685,103835,1),(82687,103837,1),(82688,103838,1),(82690,103840,1),(82691,103841,1),(82694,103844,1),(82699,103849,1),(82703,103853,1),(82706,103856,1),(82709,103861,2),(82711,103863,1),(82714,103866,1),(82715,103867,1),(82716,103868,1),(82713,103865,1),(82712,103864,1),(82553,103787,1),(82558,103681,1),(82644,103783,2),(82647,103786,1),(82648,103788,1),(82698,103848,1),(82670,103814,2),(82686,103836,2),(82707,103858,1),(82555,103817,1),(82561,103684,2),(82651,103792,1),(82695,103845,1)],(0,228630,357711,211522),[(200327,424,(),(7981,6652,7937,8828,1498,8767),()),(193001,418,(),(8836,8840,8902,8960,8783,8782),(192985,415,192948,415,192948,415)),(200329,424,(),(7981,6652,8826,1498,8767),()),(11840,1,(),(),()),(200324,421,(6625,0,0),(7981,6652,8830,1498,8767),()),(137488,421,(),(9130,7977,6652,7937,8822,8818,9144,3300,8767),()),(133610,418,(6541,0,0),(8974,7977,6652,8822,8820,9144,3297,8767),()),(193519,418,(6607,0,0),(8836,8840,8902),()),(193510,418,(6574,0,0),(8836,8840,8902,7937),()),(200326,418,(),(40,8829,8974,7977,1495,8767),()),(192999,418,(6556,0,0),(8836,8840,8902,8780),(192948,415)),(193000,418,(6556,0,0),(8836,8840,8902,8780),(192948,415)),(110007,415,(),(7977,6652,9144,8973,3300,8767),()),(193677,415,(),(7977,6652,9144,8973,1637,8767),()),(200332,415,(6592,0,0),(41,7977,8973,1498,8767),()),(201997,421,(6643,6518,0),(9130,7977,43,9147,1643,8767),()),(195513,421,(),(6652,7981,1498,8767),()),(198731,1,(),(),())],[Player-57-0CF2A300,384235,Player-57-0D5D2DEA,1459,Player-57-0CF2A300,162448,Player-57-0CF2A300,371172,Player-57-0CF2A300,396092,Player-57-0D6972FB,21562,Player-57-079A5A31,6673,Player-57-0D32803E,389684,Player-57-0D5E5609,381753],77,0,0,0";
        let mut iter = super::LineReader::new(doc.into());
        let res = iter
            .next()
            .expect("had no line")
            .expect("could not read line");

        let expected = vec![
            "4/5 18:51:38.090",
            "COMBATANT_INFO",
            "Player-57-0CF2A300",
            "0",
            "1251",
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
            "[(82710,103678,1),",
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
