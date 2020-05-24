use {fehler::throws, anyhow::Error, reqwest::Url, bytes::Bytes};

pub trait Client {
    #[throws] fn get(&self, url: Url) -> Bytes;
}

impl Client for reqwest::blocking::Client {
    #[throws] fn get(&self, url: Url) -> Bytes { self.get(url).send()?.bytes()? }
}

pub fn client() -> impl Client {
    cfg_if::cfg_if! {
        if #[cfg(feature="static_http_cache")] {
            impl Client for static_http_cache::Cache<reqwest::blocking::Client> {
                #[throws] fn get(&self, url: Url) -> File { static_http_cache::Cache<reqwest::blocking::Client>::get(&self, url).unwrap()/*fixme*/ }
            }
            lazy_static::lazy_static!{
                pub static ref cache : std::sync::Mutex<static_http_cache::Cache<reqwest::Client>> = {
                    let path = std::env::temp_dir().join("static_http_cache");
                    std::fs::DirBuilder::new().recursive(true).create(&path).unwrap();
                    std::sync::Mutex::new(static_http_cache::Cache::new(path, reqwest::Client::new()).unwrap())
                };
            }
            cache.lock().unwrap();
        } else {
            reqwest::blocking::Client::new()
        }
    }
}
