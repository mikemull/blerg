use std::error::Error;
use std::io;
use std::process;
use std::fs::File;


pub fn read_mac_file() -> Result<(), Box<dyn Error>> {
    let file = File::open("macs.csv")?;
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        println!("{:?}", record);
    }
    Ok(())
}
