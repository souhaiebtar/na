
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::SystemTime;

pub struct Directory {
    pub root: PathBuf,
}


pub struct FileMeta {
    pub name: String,
    pub size: u64,
//    pub modified: SystemTime
}


impl Directory {

    pub fn new(root: PathBuf) -> Directory {
        Directory {
            root: root,
        }
    }

    /// Get a table of uri => filename from the root directory. Files are
    /// not listed recursively, only the base level files are listed.
    ///  Directories are ommited as well.
    pub fn list_available_resources(&self) -> HashMap<String, FileMeta> {
        let mut files: HashMap<String, FileMeta> = HashMap::new();
        let paths = fs::read_dir(&(self.root)).unwrap();

        for p in paths {
            let pu = p.unwrap();
            if pu.file_type().unwrap().is_file() {
                files.insert(
                    format!("/{}", pu.file_name().into_string().unwrap()),
                    FileMeta {
                        name: format!("{}",  pu.file_name().into_string().unwrap()),
                        size: pu.metadata().unwrap().len(),
                   //     modified: pu.metadata().unwrap().modified().unwrap()
                    });
            }
        }
        files
    }

    /// Returns the full path of a file with the name "name". The file
    /// need not be an already existing file.
    pub fn full_path(&self, name: String) -> PathBuf {
        let mut path = PathBuf::new();
        path.push(self.root.to_str().unwrap());
        path.push(name);
        path
    }
}
