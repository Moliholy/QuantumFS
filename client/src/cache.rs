use std::{env, fs};
use std::path::PathBuf;

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    fn new(path: &str) -> Self {
        let cache_dir = PathBuf::from(path).join(".qfs");
        fs::create_dir_all(cache_dir.join("data"))
            .expect("Failure creating the ~/.qfs directory");
        Self {
            cache_dir,
        }
    }

    pub fn data_dir(&self) -> PathBuf {
        self.cache_dir.join("data")
    }

    pub fn main_dir(&self) -> PathBuf {
        self.cache_dir.clone()
    }
}


lazy_static! {
    pub static ref CACHE: Cache = Cache::new(env::var("HOME").unwrap().as_str());
}
