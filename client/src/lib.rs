use {fehler::throws, anyhow::{Error, ensure}};
pub use reqwest::Url;

struct Cache {
    cache: static_http_cache::Cache<reqwest::blocking::Client>,
    query_count: std::collections::HashMap<String, u32>,
}
impl Cache {
    fn new() -> Self {
        Self{cache: static_http_cache::Cache::new(dirs::cache_dir().unwrap().join("static_http_cache"), reqwest::blocking::Client::new()).unwrap(),
                query_count: Default::default()}
    }
}

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
impl Client for Cache {
   #[throws] fn get(&mut self, url: Url) -> Vec<u8> {
        let host = url.host_str().unwrap_or_default();
        let query_count = self.query_count.entry(host.to_owned()).or_default();
        ensure!(*query_count < 3 || host == "router.project-osrm.org", "{:?}", self.query_count);
        *query_count += 1;
        Client::get(&mut self.cache, url)?
    }
}
impl<T:Client> Client for std::sync::MutexGuard<'_, T> {
    #[throws] fn get(&mut self, url: Url) -> Vec<u8> { std::ops::DerefMut::deref_mut(self).get(url)? }
}

pub fn client() -> impl Client {
    #![allow(non_upper_case_globals)]
    lazy_static::lazy_static!{ static ref cache : std::sync::Mutex<Cache> = std::sync::Mutex::new(Cache::new()); }
    cache.lock().unwrap()
}
