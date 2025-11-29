use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct OutDir {
    path: PathBuf,
}

impl OutDir {
    pub fn new(path: PathBuf) -> Self {
        fs::create_dir_all(&path).unwrap();
        OutDir { path }
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        let path = self.path.join(path);
        fs::create_dir_all(&path).unwrap();
        OutDir { path }
    }

    // pub fn compressed_file_writer(&self, file_name: &str) -> GzEncoder<File> {
    //     let file = File::create(self.path.join(file_name)).unwrap();
    //     GzEncoder::new(file, Compression::best())
    // }

    pub fn create_compressed_file(&self, file_name: &str, data: &[u8]) {
        let path = self.path.join(file_name);
        std::fs::write(path, data).unwrap();

        // TODO: does not compress yet... it actually makes images take up more space, so need to filter images out of compression.
        // let file = File::create(&path).unwrap();
        // let mut writer = GzEncoder::new(file, Compression::best());
        // writer.write_all(data).unwrap();

        // TODO
        // Path::new("/")
        //     .join(path.strip_prefix(self.root).unwrap())
        //     .into_os_string()
        //     .into_string()
        //     .unwrap()
    }
}
