use {fehler::throws, anyhow::Error};
pub use reqwest::Url;

pub trait Client {
    #[throws] fn get(&mut self, url: Url) -> Vec<u8>;
}

impl Client for static_http_cache::Cache<reqwest::blocking::Client> {
    #[throws] fn get(&mut self, url: Url) -> Vec<u8> {
        use std::io::Read;
        let mut buffer = Vec::new();
        static_http_cache::Cache::<reqwest::blocking::Client>::get(self, url)?.read_to_end(&mut buffer)?;
        buffer
    }
}
impl<T:Client> Client for std::sync::MutexGuard<'_, T> {
    #[throws] fn get(&mut self, url: Url) -> Vec<u8> { std::ops::DerefMut::deref_mut(self).get(url)? }
}

pub fn client() -> impl Client {
    #![allow(non_upper_case_globals)]
    lazy_static::lazy_static!{
        static ref cache : std::sync::Mutex<static_http_cache::Cache<reqwest::blocking::Client>> = {
            let path = dirs::cache_dir().unwrap().join("static_http_cache");
            std::fs::DirBuilder::new().recursive(true).create(&path).unwrap();
            std::sync::Mutex::new(static_http_cache::Cache::new(path, reqwest::blocking::Client::new()).unwrap())
        };
    }
    cache.lock().unwrap()
}
