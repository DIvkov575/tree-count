use std::collections::HashMap;
use std::ffi::{OsString};
use std::fmt::{Debug, Display, Error, Formatter};
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use prettytable::row;
use clap::{self, Parser};


#[derive(Debug)]
struct Table {
    map: HashMap<OsString, u64>
}
impl Table {
    fn new() -> Self {
        Self {map: HashMap::new()}
    }

    fn print_horizontal(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut table = prettytable::Table::new();
        let mut names = Vec::with_capacity(self.map.capacity());
        let mut sizes = Vec::with_capacity(self.map.capacity());

        let mut intermediary = self.map.iter().collect::<Vec<(&OsString, &u64)>>();
        intermediary.sort_by(|(x1, _), (x2, _)| x1.cmp(x2));
        intermediary.iter().for_each(|row| {
            let name = Self::format_name(&row);
            let size = Self::format_size(&row);
            names.push(name);
            sizes.push(size);
        });

        table.add_row(names.into());
        table.add_row(sizes.into());

        write!(f, "{}", table.to_string())?;

        Ok(())
    }

    fn print_vertical(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut table = prettytable::Table::new();
        let mut intermediary = self.map.iter().collect::<Vec<(&OsString, &u64)>>();
        intermediary.sort_by(|(x1, _), (x2, _)| x1.cmp(x2));
        intermediary.iter().for_each(|row| {
            let name = Self::format_name(row);
            let size = Self::format_size(row);
            table.add_row(row![name, size]);
        });

        write!(f, "{}", table.to_string())?;

        Ok(())
    }

    fn format_name(row: &(&OsString, &u64)) -> String {
        let name = (format!(".{}", row.0.to_str().unwrap()));
        name
    }

    fn format_size(row: &(&OsString, &u64)) -> String {
        let size = match u64::ilog10(row.1.to_owned()) / 3 {
            0 => { format!("{}", row.1) }
            1 => { format!("{}K", row.1 / 1000) }
            2 => { format!("{}M", row.1 / 1000_000) }
            _ => { format!("{}G", row.1 / 1000_000_000) }
        };
        size
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.print_vertical(f)
    }
}

#[derive(PartialEq, Debug)]
enum Mode {
    FileCount,
    CharCount,
    LineCount,
}


#[derive(Parser, Debug)]
#[command(version, about, long_about = None,)]
struct Args {
    #[arg(long, short)]
    file_count: bool,
    #[arg(long, short)]
    char_count: bool,
    #[arg(long, short)]
    line_count: bool,
}


fn main() {
    let mut mode;
    let args = Args::parse();
    if args.file_count {
        mode = Mode::FileCount;
    } else if args.char_count {
        mode = Mode::CharCount;
    } else if args.line_count {
        mode = Mode::LineCount;
    } else {
        mode = Mode::CharCount
    }

    let mut table = Table::new();

    let mut files: Vec<PathBuf> = Vec::new();
    recursive_walk(Path::new("."),  &mut files);

    for file in &files {
        if let Some(extension) = file.extension() {
            if let Some(v) = table.map.get_mut(extension) {
                if mode == Mode::CharCount {
                    *v += file.metadata().unwrap().len();
                } else if mode == Mode::FileCount{
                    *v += 1;
                } else if mode == Mode::LineCount {
                    *v += BufReader::new(File::open(file).unwrap()).lines().count() as u64;
                }
            } else {
                if mode == Mode::CharCount{
                    table.map.insert(extension.to_owned().to_owned(), file.metadata().unwrap().len());
                } else if mode == Mode::FileCount{
                    table.map.insert(extension.to_owned().to_owned(), 1);
                } else if mode == Mode::LineCount {
                    table.map.insert(extension.to_owned().to_owned(), BufReader::new(File::open(file).unwrap()).lines().count() as u64);
                }

            }
        }
    }

    println!("{:?}", mode);
    println!("{}", table);

}
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
