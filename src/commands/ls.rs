use std::fmt::{Formatter, Result, Display};
use std::fs::{self, Permissions};
use std::os::unix::prelude::{PermissionsExt, MetadataExt};
use std::path::{PathBuf, Path};
use std::error::Error;
use std::result;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use clap::Parser;
use nix::unistd::{User, Uid, Group, Gid};
use regex::Regex;

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


struct FileData {
    path: PathBuf,
    permissions: Permissions,
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
            "{} {:>4} {} {}  {:>9}  {} {}", 
            parse_permissions(self.permissions.clone()), 
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

        let is_hidden_file: Regex = Regex::new(r"(^|\/)\.[^\/\.]").unwrap();
       
       for entry in fs::read_dir(path)? { 
            let file: PathBuf = entry.unwrap().path(); 
            
            // if there is hidden folder in directory do not show it unless option is toggled
            if args.all || !is_hidden_file.is_match(file.to_str().unwrap()) {
                files.push(
                    get_filedata(&file)
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
            permissions: meta.permissions(),
            date_modified: meta.modified().unwrap(),
            hard_link_count: meta.nlink(),
            owner: User::from_uid( Uid::from_raw( meta.uid() ) ).unwrap().unwrap().name,
            group: Group::from_gid( Gid::from_raw(meta.gid())).unwrap().unwrap().name,
        };
    }

    panic!("could not get file metadata");

    
}




fn parse_permissions(permissions: Permissions) -> String {
    let permissions_u32 = permissions.mode() & 511;

    let mut all_permissions_u32: [u32; 3] = [0, 0 , 0];
    let mut i = 0;
    let len = all_permissions_u32.len();

    while i < len {
        // permission is of form 110100100 
        all_permissions_u32[i] = (permissions_u32 >> (len * (2 - i))) & 7;
        i += 1;
    }

    let mut permission_symbols: String = "".to_string();

    for permission in all_permissions_u32.iter() {
        permission_symbols.push_str(&to_permission_symbol(permission.to_owned()));
    }

    return permission_symbols;


}

fn to_permission_symbol(permission: u32) -> String {
	
    let mut permission_symbols: [char; 3] = ['r', 'w', 'x'];

    let mut i: usize = 0;
    let len: usize = permission_symbols.len();

    while i < len {

        let offset: u32 = (len - 1 - i).try_into().unwrap();
        let val: u32 = u32::pow(2, offset);
        
        if permission & val != val {
            permission_symbols[i] = '-';
        }

        i += 1;

    }

    return permission_symbols.iter().collect();
    
}

fn parse_date_modified(date: SystemTime) -> String {
    return DateTime::<Local>::from(date).format("%b %d %H:%M").to_string();
}

