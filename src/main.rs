use std::any::Any;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Display, Formatter};
use std::fs::{File, read_dir};
use std::hash::Hash;
use std::io::Read;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::process::exit;
use nix::libc::printf;

// section table
#[derive(Debug)]
struct Table {
    map: HashMap<OsString, u64>
}
impl Table {
    fn new() -> Self {
        Self {map: HashMap::new()}
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut names = Vec::with_capacity(self.map.capacity());
        let mut sizes= Vec::with_capacity(self.map.capacity());

        let mut intermediary = self.map.iter().collect::<Vec<(&OsString, &u64)>>();
        // intermediary.sort_by(|(_,y1), (_,y2)| y2.cmp(y1));
        intermediary.sort_by(|(x1,_), (x2,_)| x1.cmp(x2));

        for row in intermediary {
            names.push(format!(".{}", row.0.to_str().unwrap()));

            let size = match u64::ilog10(row.1.to_owned()) / 3 {
                0 => { format!("{}", row.1) }
                1 => { format!("{}K", row.1/1000) }
                2 => { format!("{}M", row.1/1000_000) }
                _ => { format!("{}G", row.1/1000_000_000) }
            };
            sizes.push(size);
        }

        let mut table = prettytable::Table::new();
        table.add_row(names.into());
        table.add_row(sizes.into());

        write!(f, "{}", table.to_string())?;

        Ok(())
    }
}

//section mode
#[derive(PartialEq)]
enum Mode {
    FileCount,
    CharCount,
    LineCount,
}

//section main
fn main() {

    let mode = Mode::CharCount;
    let mut table = Table::new();

    let mut files: Vec<PathBuf> = Vec::new();
    recursive_walk(Path::new("."),  &mut files);

    for file in &files {
        if let Some(extension) = file.extension() {
            if let Some(v) = table.map.get_mut(extension) {
                if mode == Mode::CharCount {
                    *v += file.metadata().unwrap().len();
                } else if mode == Mode::FileCount{
                    *v += 1
                } else if mode == Mode::LineCount {
                    read_lines()
                    // *v += File::from(file).read_vectored()
                }
            } else {
                if mode == Mode::CharCount{
                    table.map.insert(extension.to_owned().to_owned(), file.metadata().unwrap().len());
                } else if mode == Mode::FileCount{
                    table.map.insert(extension.to_owned().to_owned(), 1);
                }

            }
        }
    }

    println!("{}", table);



}
//section recursive
fn recursive_walk(dir: &Path, files: &mut Vec<PathBuf>) {
    for file in read_dir(dir).unwrap() {
        let path = file.unwrap().path();
        if path.is_dir() {
            recursive_walk(&path, files);
        } else {
            files.push(path);
        }
    }
}
