use std::{
    fs::File,
    io,
    io::{stdout, BufRead, BufReader},
    process::exit,
};

fn do_file(r: impl std::io::Read, w: impl std::io::Write) -> std::io::Result<()> {
    let bufr = BufReader::new(r);
    for line in bufr.lines() {
        println!("{}", line?);
    }

    todo!()
}

fn count_max_columns(r: impl std::io::Read) -> io::Result<usize> {
    BufReader::new(r).lines().try_fold(0, |prev, line| {
        // There's one extra column because the first column will be both timestamp and event.
        // TODO: I expect this could be much faster if we counted the incidences of , in the string rather than splitting it.
        let n = line?.split(',').count() + 1;
        Ok(if prev > n { prev } else { n })
    })
}

fn main() {
    let out = stdout();
    let args = std::env::args().skip(1);
    let max_columns: std::io::Result<usize> =
        std::env::args()
            .skip(1)
            .map(File::open)
            .try_fold(0, |prev, file| {
                let n = count_max_columns(file?)?;
                Ok(if prev > n { prev } else { n })
            });

    println!("max cols: {:?}", max_columns);

    // Open files once to count the maximum number of columns.
    // for name in args {
    //     if let Err(error) = File::open(&name).and_then(|mut handle| do_file(&mut handle, &out)) {
    //         eprintln!("could not process {}: {}", name, error);
    //         exit(error.raw_os_error().unwrap_or(-1));
    //     }
    // }
    // Open them again to write out the CSV in the correct format with the correct number of columns.

    println!("Hello, world!");
}
