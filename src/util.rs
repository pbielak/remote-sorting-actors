/*
Util functions
*/
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::fs::File;
use std::path::PathBuf;


pub fn read_numbers(path: PathBuf) -> Result<Vec<i64>, Error> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| {
            line.and_then(|v| {
                v
                    .parse()
                    .map_err(|e| {
                        Error::new(ErrorKind::InvalidData, e)
                    })
            })
        })
        .collect()
}


pub fn write_numbers(path: PathBuf, numbers: &Vec<i64>) -> Result<bool, Error> {
    let mut file = File::create(path).unwrap();

    numbers.iter().for_each(|v| {
        file.write_all(format!("{}\n", v).as_bytes()).unwrap()
    });


    Ok(true)
}
