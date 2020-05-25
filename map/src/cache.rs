//pub trait Client = std::ops::DerefMut<Target=static_http_cache::Cache<reqwest::blocking::Client>>;

use {fehler::throws, anyhow::Error, reqwest::Url, bytes::Bytes};

pub trait Client {
    #[throws] fn get(&mut self, url: Url) -> Bytes;
}

/*impl Client for reqwest::blocking::Client {
    #[throws] fn get(&mut self, url: Url) -> Bytes { reqwest::blocking::Client::get(&self, url).send()?.bytes()? }
}*/

impl Client for static_http_cache::Cache<reqwest::blocking::Client> {
    #[throws] fn get(&mut self, url: Url) -> Bytes {
        use std::io::Read;
        let mut bytes = Vec::new();
        static_http_cache::Cache::<reqwest::blocking::Client>::get(self, url)?.read_to_end(&mut bytes)?;
        bytes.into()
    }
}
impl<T:Client> Client for std::sync::MutexGuard<'_, T> {
    #[throws] fn get(&mut self, url: Url) -> Bytes { std::ops::DerefMut::deref_mut(self).get(url)? }
}

pub fn client() -> impl Client {
    lazy_static::lazy_static!{
        static ref cache : std::sync::Mutex<static_http_cache::Cache<reqwest::blocking::Client>> = {
            let path = dirs::cache_dir().unwrap().join("static_http_cache");
            std::fs::DirBuilder::new().recursive(true).create(&path).unwrap();
            std::sync::Mutex::new(static_http_cache::Cache::new(path, reqwest::blocking::Client::new()).unwrap())
        };
    }
    cache.lock().unwrap()
}
