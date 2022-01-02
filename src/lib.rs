use crc::{Crc, CRC_32_ISCSI};
use rusqlite::{params, Connection, Error};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt, Debug)]
#[structopt(name = "multi-machine-dedup")]
pub struct CLI {
    #[structopt(subcommand)]
    pub cmd: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    #[structopt(name = "index", about = "Use index to create or update a database")]
    Index(IndexOptions),
    #[structopt(
        name = "check-integrity",
        about = "Use check-integrity to verify all files"
    )]
    CheckIntegrity(CheckIntegrityOptions),
}

#[derive(StructOpt, Debug)]
pub struct IndexOptions {
    #[structopt(short, long)]
    pub host: String,
    #[structopt(short, long, parse(from_os_str))]
    pub db: PathBuf,
    pub path: PathBuf,
}

#[derive(StructOpt, Debug)]
pub struct CheckIntegrityOptions {
    #[structopt(short, long)]
    pub host: String,
    #[structopt(short, long, parse(from_os_str))]
    pub db: PathBuf,
}

#[derive(Clone, Debug)]
struct Entry {
    hash: u32,
    full_path: String,
    size: u64,
    mime: String,
}

pub const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

pub fn create_db(conn: &Connection) {
    //id              INTEGER PRIMARY KEY AUTOINCREMENT,
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file (
                  host TEXT,
                  full_path TEXT,
                  hash              INTEGER,
                  size INTEGER,              
                  PRIMARY KEY (host, full_path)
                  FOREIGN KEY(hash,size) REFERENCES hash(hash,size)
                  )",
        [],
    )
    .expect("");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS hash (
                  hash              INTEGER,
                  size INTEGER,
                  mime TEXT,
                  PRIMARY KEY (hash, size)
                  )",
        [],
    )
    .expect("");
}

pub fn index(opt: IndexOptions) {
    //let conn = Connection::open_in_memory().unwrap();
    let conn = Connection::open(opt.db).unwrap();
    create_db(&conn);
    for entry in WalkDir::new(opt.path) {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            println!("Processsing directory {}", entry.path().display());
        } else {
            let x = fs::metadata(entry.path()).unwrap().len();
            println!("file {} {:?}", entry.path().display(), x);
            let crc = hash(entry.path());
            println!("The crc is: {} for file {}", crc, entry.path().display());
            let data = Entry {
                hash: crc,
                full_path: entry.path().display().to_string(),
                size: fs::metadata(entry.path()).unwrap().len(),
                mime: tree_magic_mini::from_filepath(entry.path())
                    .get_or_insert("N/A")
                    .to_string(),
            };
            let res = conn.execute(
                "INSERT INTO hash (hash, size, mime) VALUES (?1, ?2, ?3)",
                params![data.hash, data.size, data.mime],
            ); //.expect("req1");
            match res {
                Ok(_) => (),
                Err(error) => match error {
                    Error::SqliteFailure(error, _msg) => {
                        if error.extended_code == 1555 {
                            println!(
                                "hash & size '{}' '{}' already indexed",
                                data.hash, data.size
                            )
                        }
                    }
                    _ => panic!(
                        "Unable to index hash & size: '{}' {}",
                        data.full_path, error
                    ),
                },
            }
            let res = conn.execute(
                "INSERT INTO file (host, full_path, hash, size) VALUES (?1, ?2, ?3, ?4)",
                params![opt.host, data.full_path, data.hash, data.size],
            ); //.expect("req2");
            match res {
                Ok(_) => (),
                Err(error) => match error {
                    Error::SqliteFailure(error, _msg) => {
                        if error.extended_code == 1555 {
                            println!("path '{}' already indexed", data.full_path)
                        }
                    }
                    _ => panic!("Unable to index file: '{}' {}", data.full_path, error),
                },
            }
        }
    }
}

pub fn check_integrity(opt: CheckIntegrityOptions) {
    let conn = Connection::open(opt.db).unwrap();
    let mut stmt = conn.prepare("SELECT * FROM file WHERE host=:host").unwrap();
    let file_iter = stmt
        .query_map(&[(":host", &opt.host)], |row| {
            Ok(Entry {
                hash: row.get(2)?,
                full_path: row.get(1)?,
                size: row.get(3)?,
                mime: "".to_string(),
            })
        })
        .unwrap();

    for file in file_iter {
        let stored_hash: u32 = file.as_ref().unwrap().hash;
        let path = &file.as_ref().unwrap().full_path;
        if stored_hash != hash(&PathBuf::from(path)) {
            println!("check failed on file: '{}'", path);
        }
    }
}

fn get_file_content(path: &Path) -> Vec<u8> {
    let mut buffer = BufReader::new(File::open(path).unwrap());
    let mut file_content = Vec::new();
    let _ = buffer.read_to_end(&mut file_content);
    file_content
}

fn hash(path: &Path) -> u32 {
    let mut digest = CASTAGNOLI.digest();
    let current_file_content = get_file_content(path);
    digest.update(&current_file_content);
    digest.finalize()
}
