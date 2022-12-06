use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use lazy_static::lazy_static;
use regex::{Regex, RegexSet};

static SUPPORTED_FILE_FORMATS: &[&str] = &["4"];
lazy_static! {
        static ref RE_SET: regex::RegexSet = RegexSet::new([
            LineRegex::Version.regex_str(),
            LineRegex::Format.regex_str(),
            LineRegex::Start.regex_str(),
            LineRegex::FunctionEntry.regex_str(),
            LineRegex::FunctionExit.regex_str(),
            LineRegex::Penultimate.regex_str(),
            LineRegex::End.regex_str(),
        ])
        .unwrap();
    }
#[derive(Clone, Debug)]
enum RecType {
    Entry,
    Exit,
    Format,
    StartTime,
    Version,
}

trait XtraceRecord {
    fn new(line: &str) -> Self;
}

trait XtraceFn {}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct XtraceFileRecord {
    pub id: uuid::Uuid,
    pub start: Option<XtraceStartTimeRecord>,
    pub format: Option<XtraceFmtRecord>,
    pub version: Option<XtraceVersionRecord>,
    pub fn_records: Vec<XtraceFnRecord>,
}

impl XtraceFileRecord {
    fn add_fn_record(&mut self, func: XtraceFnRecord) {
        self.fn_records.push(func);
    }

    pub fn print_tree(&self) {
        for record in self.fn_records.iter() {
            if let Some(entry_record) = &record.entry_record {
                let prefix = "  ".repeat(entry_record.level.try_into().unwrap());
                println!(
                    "{prefix}{}({}) ({}) ({})",
                    &entry_record.fn_name,
                    &entry_record.fn_type,
                    &entry_record.file_name,
                    &entry_record.inc_file_name
                );
            }
        }
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct XtraceFnRecord {
    pub fn_num: u32,
    pub entry_record: Option<XtraceEntryRecord>,
    pub exit_record: Option<XtraceExitRecord>,
    //return_record: Option<XtraceReturnRecord>,
}

impl XtraceRecord for XtraceVersionRecord {
    fn new(line: &str) -> Self {
        let re = Regex::new(LineRegex::Version.regex_str()).unwrap();
        let cap = re.captures(line).unwrap();
        let version = cap
            .name("version")
            .expect("version number not found")
            .as_str()
            .to_owned();
        XtraceVersionRecord {
            version,
            rec_type: RecType::Version,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct XtraceVersionRecord {
    pub version: String,
    rec_type: RecType,
}

impl XtraceRecord for XtraceStartTimeRecord {
    fn new(line: &str) -> Self {
        let re = Regex::new(LineRegex::Start.regex_str()).unwrap();
        let _cap = re.captures(line).ok_or("oops").unwrap();
        XtraceStartTimeRecord {
            start_time: String::from("Sat Dec  3 18:01:30 PST 2022"),
            rec_type: RecType::StartTime,
        }
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct XtraceStartTimeRecord {
    pub start_time: String,
    rec_type: RecType,
}

impl XtraceRecord for XtraceFmtRecord {
    fn new(line: &str) -> Self {
        let re = Regex::new(LineRegex::Format.regex_str()).unwrap();
        let cap = re.captures(line).ok_or("oops").unwrap();
        let format = cap
            .name("format")
            .expect("version number not found")
            .as_str();
        if SUPPORTED_FILE_FORMATS.contains(&format) {
            XtraceFmtRecord {
                format: format
                    .parse::<u32>()
                    .expect("Unable to parse format number into an integer"),
                rec_type: RecType::Format,
            }
        } else {
            panic!("Unsupported version: {}", format);
        }
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct XtraceFmtRecord {
    pub format: u32,
    rec_type: RecType,
}

/*    enum FnType {
    Internal,
    User,
}*/

impl XtraceFn for XtraceEntryRecord {}
impl XtraceRecord for XtraceEntryRecord {
    fn new(line: &str) -> Self {
        let re = Regex::new(LineRegex::FunctionEntry.regex_str()).unwrap();
        let cap = re.captures(line).ok_or("oops").unwrap();
        return XtraceEntryRecord {
            rec_type: RecType::Entry,
            level: cap.name("level").unwrap().as_str().parse::<u32>().unwrap(),
            fn_num: cap.name("fn_num").unwrap().as_str().parse::<u32>().unwrap(),
            time_idx: cap
                .name("time_idx")
                .unwrap()
                .as_str()
                .parse::<f64>()
                .unwrap(),
            mem_usage: cap
                .name("mem_usage")
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap(),
            fn_name: cap.name("fn_name").unwrap().as_str().to_owned(),
            fn_type: cap.name("fn_type").unwrap().as_str().parse::<u8>().unwrap(),
            inc_file_name: cap.name("inc_file_name").unwrap().as_str().to_owned(),
            file_name: cap.name("file_name").unwrap().as_str().to_owned(),
            line_num: cap
                .name("line_num")
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap(),
            arg_num: cap
                .name("arg_num")
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap(),
            args: cap.name("args").unwrap().as_str().to_owned(),
        };
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct XtraceEntryRecord {
    rec_type: RecType,
    pub level: u32,
    pub fn_num: u32,
    pub time_idx: f64,
    pub mem_usage: u32,
    pub fn_name: String,
    pub fn_type: u8,
    pub inc_file_name: String,
    pub file_name: String,
    pub line_num: u32,
    pub arg_num: u32,
    //TODO Make this a byte slice
    pub args: String,
}

impl XtraceFn for XtraceExitRecord {}
impl XtraceRecord for XtraceExitRecord {
    fn new(line: &str) -> Self {
        let re = Regex::new(LineRegex::FunctionExit.regex_str()).unwrap();
        let cap = re.captures(line).ok_or("oops").unwrap();
        XtraceExitRecord {
            rec_type: RecType::Exit,
            level: cap.name("level").unwrap().as_str().parse::<u32>().unwrap(),
            fn_num: cap.name("fn_num").unwrap().as_str().parse::<u32>().unwrap(),
            time_idx: cap
                .name("time_idx")
                .unwrap()
                .as_str()
                .parse::<f64>()
                .unwrap(),
            mem_usage: cap
                .name("mem_usage")
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap(),
        }
    }
}
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct XtraceExitRecord {
    pub level: u32,
    pub fn_num: u32,
    rec_type: RecType,
    pub time_idx: f64,
    pub mem_usage: u32,
}

enum LineRegex {
    Version,
    Format,
    Start,
    FunctionEntry,
    FunctionExit,
    End,
    Penultimate,
}

impl LineRegex {
    fn regex_str(&self) -> &str {
        match self {
            LineRegex::Version => r"Version:\s+(?P<version>\d+\.\d+\.\d+).*",
            LineRegex::Format => r"^File format: (?P<format>\d+)",
            LineRegex::Start => {
                r"^TRACE START \[(?P<start>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}.\d+)\]"
            }
            LineRegex::FunctionEntry => {
                r"^(?P<level>\d+)\t(?P<fn_num>\d+)\t(?P<rec_type>0)\t(?P<time_idx>\d+\.\d+)\t(?P<mem_usage>\d+)\t(?P<fn_name>.*)\t(?P<fn_type>[01])\t(?P<inc_file_name>.*)\t(?P<file_name>.*)\t(?P<line_num>\d+)\t(?P<arg_num>\d+)\t?(?P<args>.*)"
            }
            LineRegex::FunctionExit => {
                r"^(?P<level>\d+)\t(?P<fn_num>\d+)\t(?P<rec_type>1)\t(?P<time_idx>\d+\.\d+)\t(?P<mem_usage>\d+).*"
            }
            LineRegex::Penultimate => r"^\s+(?P<time_idx>\d+\.\d+)\t(?P<mem_usage>\d+)",
            LineRegex::End => {
                r"^TRACE END\s+\[(?P<end>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}.\d+)\]"
            }
        }
    }
}

fn process_line(
    run: &mut XtraceFileRecord,
    entry_cache: &mut HashMap<u32, XtraceEntryRecord>,
    line: &String,
) {
    let matches: Vec<_> = RE_SET.matches(line.as_str()).into_iter().collect();
    if matches.is_empty() {
        eprintln!("No matches for line: {line}");
        return;
    }
    let idx = matches.first().unwrap();
    match idx {
        0 => run.version = Some(XtraceVersionRecord::new(line)),
        1 => run.format = Some(XtraceFmtRecord::new(line)),
        2 => run.start = Some(XtraceStartTimeRecord::new(line)),
        3 => {
            let record = XtraceEntryRecord::new(line);
            entry_cache.insert(record.fn_num, record);
        }
        4 => {
            let exit_record = XtraceExitRecord::new(line);
            if let Some(entry_record) = entry_cache.get(&exit_record.fn_num) {
                let fn_record = XtraceFnRecord {
                    fn_num: exit_record.fn_num,
                    entry_record: Some(entry_record.to_owned()),
                    exit_record: Some(exit_record),
                };
                run.add_fn_record(fn_record);
            }
        }
        5 => {}
        6 => {}
        _ => todo!(),
    };
}

pub fn parse_xtrace_file(
    id: uuid::Uuid,
    file: String,
) -> Result<XtraceFileRecord, std::io::Error> {
    let xtrace_file = File::open(file)?;
    let mut reader = BufReader::new(xtrace_file);
    //let mut line = String::new();
    let mut line: Vec<u8> = Vec::new();
    let mut file_run = XtraceFileRecord {
        id,
        format: None,
        start: None,
        version: None,
        fn_records: Vec::new(),
    };
    let mut entry_cache: HashMap<u32, XtraceEntryRecord> = HashMap::new();
    let mut line_number: u32 = 1;
    loop {
        //let result = reader.read_line(&mut line);
        let result = reader.read_until(0xA, &mut line);
        match result {
            Ok(size) => {
                if size == 0 {
                    return Ok(file_run);
                }
                //println!("Processing line {line_number}: {line}");
                process_line(
                    &mut file_run,
                    &mut entry_cache,
                    &String::from_utf8_lossy(line.as_slice()).to_string(),
                );
            }
            Err(e) => {
                eprintln!("Error reading line #{line_number}: {e}");
                continue;
            }
        }
        line_number += 1;
        line.clear();
    }
}

// Not yet implemented
/*    struct XtraceReturnRecord {
    level: u32,
    fn_num: u32,
    rec_type: RecType,
    ret_val: u32, // Need to confirm this type. I have yet to see an example to work from and the docs aren't specific.
}*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
