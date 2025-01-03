use crate::trm::LOG_FILE;

use chrono::{DateTime, Local};
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Error, Write},
};

#[derive(Serialize, Deserialize, Debug)]
pub enum OpType {
    TRASH,
    RESTORE,
}

impl OpType{
    pub fn to_string(&self) -> String{
        match &self{
            Self::TRASH => String::from("Trash"),
            Self::RESTORE => String::from("Restore")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    /// The original path from where the path was moved
    pub src: Vec<String>,

    /// The path where the file ended up in  
    pub dst: Vec<String>,

    /// Type of operation
    pub operation: OpType,

    /// The datetime when it was moved
    pub moved_time: DateTime<Local>,
}

pub fn append_to_logs(info: &FileInfo) -> Result<(), Error> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(LOG_FILE)
        .unwrap();

    let mut writer = io::BufWriter::new(file);
    let serialized_info = serde_json::to_string(info)?;
    writeln!(writer, "{}", serialized_info)?;
    writer.flush()?;
    Ok(())
}

pub fn read_all_logs() {
    let file = File::open(LOG_FILE).unwrap();
    let reader = BufReader::new(file);
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Time", "Operation", "Source", "Destination"]);
    

    for line in reader.lines() {
        let line = line.unwrap();
        if let Ok(log) = serde_json::from_str::<FileInfo>(&line) {
            table.add_row(vec![
                log.moved_time.to_rfc2822(),
                log.operation.to_string(),
                log.src.join("\n"),
                log.dst.join("\n")
                ]
            );
        }
    }

    println!("{}", table);
}
