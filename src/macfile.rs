use std::error::Error;
use std::fs::File;
use std::str::FromStr;

use std::collections::HashMap;
use pnet::util::MacAddr;

pub fn read_mac_file() -> Result<HashMap<MacAddr, String>, Box<dyn Error>> {

    let mut mac_map: HashMap<MacAddr, String> = HashMap::new();

    let file = File::open("macs.csv")?;
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the
        // error here.
        let record = result?;
        mac_map.insert(MacAddr::from_str(&record[0]).unwrap(), record[1].to_string());
    }
    Ok(mac_map)
}

/*
    let def_mac_map: HashMap<MacAddr, &str> =
    [(MacAddr::from_str("08:9e:08:f9:30:e6").unwrap(), "Node 1"),
    (MacAddr::from_str("ac:bc:32:97:f0:bf").unwrap(), "Mac"),
    (MacAddr::from_str("08:9e:08:f9:30:e8").unwrap(), "Node 2"),
    (MacAddr::from_str("60:01:94:4c:f6:18").unwrap(), "ESP_4CF618"),
    (MacAddr::from_str("5c:cf:7f:53:31:22").unwrap(), "ESP_533122"),
    (MacAddr::from_str("42:a8:1e:fd:1c:93").unwrap(), "Em-IPhone11"),
    (MacAddr::from_str("34:15:13:25:8e:52").unwrap(), "Envoy")
    ]
    .iter().cloned().collect();
 */