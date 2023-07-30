use std::fmt::{Formatter, Result, Display};
use std::fs::{self};
use std::os::unix::prelude::MetadataExt;
use std::path::{PathBuf, Path};
use std::error::Error;
use std::{result, u32};
use std::time::SystemTime;
use chrono::{DateTime, Local};
use clap::Parser;
use nix::unistd::{User, Uid, Group, Gid};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Ls {
    #[arg(default_value = "./")]
    path: PathBuf,

    #[arg(short, default_value_t = false)]
    long: bool,

    #[arg(short, default_value_t = false)]
    all: bool,

    #[arg(short, default_value_t = false)]
    dir_not_recursive: bool,
}

struct FilePermissions {
    owner: u32,
    group: u32,
    other: u32,
    // format: u32
}

impl FilePermissions {
    fn from_permission_mode(mode: u32) -> FilePermissions {

        let permissions_u32 = mode & 511;
        
        // from for e.g. 110100111 get owner, group, other permissions
        FilePermissions {
            owner: ( permissions_u32 >> 6 ) & 7,
            group: ( permissions_u32 >> 3 ) & 7,
            other: permissions_u32 & 7,
        }
    }


    fn to_symbol_single(permission_bits: u32) -> String {
        let mut permission_symbols = ['r', 'w', 'x'];

        for (i, symbol) in permission_symbols.iter_mut().enumerate() {
            
            let relevant_bit = u32::pow(2, i.try_into().unwrap());
            
            if permission_bits & relevant_bit != relevant_bit {
                *symbol = '-';
            }

        }

        permission_symbols.iter().collect()

    }
    
}

struct FileData {
    path: PathBuf,
    permissions: FilePermissions,
    date_modified: SystemTime,
    hard_link_count: u64,
    owner: String,
    group: String,
    size: u64,
    
}

impl Display for FileData {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f, 
            "{}{}{} {:>4} {} {}  {:>9}  {} {}", 
            FilePermissions::to_symbol_single(self.permissions.owner),
            FilePermissions::to_symbol_single(self.permissions.group),
            FilePermissions::to_symbol_single(self.permissions.other),
            self.hard_link_count,
            self.owner,
            self.group,
            self.size, 
            parse_date_modified(self.date_modified), 
            self.path.to_str().unwrap()
        )
    }
}

pub fn ls(args: Ls) -> result::Result<(), Box<dyn Error>> {
    let path: PathBuf = args.path;
    let mut files: Vec<FileData> = Vec::new();

    if path.is_dir() && !args.dir_not_recursive {

       for entry in fs::read_dir(path)? {
            let file_path = entry.as_ref().unwrap().path();
            let file_name = entry.unwrap().file_name();
            
            
            // if there is hidden folder in directory do not show it unless option is toggled
            if args.all || !file_name.to_str().unwrap().starts_with('.') {
                files.push(
                    get_filedata(&file_path)
                );
            }
       }
    }
    else { 
        files.push(
            get_filedata(&path)
        );
       
    }

    for file in files {

        if args.long {
            println!("{}", file);
        } else {
            println!("{}", file.path.to_str().unwrap());
        }
    
    }
    
    Ok(())
}

fn get_filedata(path: &Path) -> FileData {
    if let Ok(meta) = fs::metadata(path) {
        return FileData {
            path: path.to_path_buf(),
            size: meta.len(),
            permissions: FilePermissions::from_permission_mode(meta.mode()),
            date_modified: meta.modified().unwrap(),
            hard_link_count: meta.nlink(),
            owner: User::from_uid( Uid::from_raw( meta.uid() ) ).unwrap().unwrap().name,
            group: Group::from_gid( Gid::from_raw(meta.gid())).unwrap().unwrap().name,
        };
    }

    panic!("could not get file metadata");
    
}

// Convert system time to proper date time format
fn parse_date_modified(date: SystemTime) -> String {
    return DateTime::<Local>::from(date).format("%b %d %H:%M").to_string();
}

