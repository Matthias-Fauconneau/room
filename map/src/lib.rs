#![allow(non_upper_case_globals)]
mod osrm;
pub use osrm::Coordinate;
use {fehler::throws, itertools::Itertools, reqwest::Url};

//#[persistentcache::persistent_cache]#[params(FileStorage, dirs::cache_dir().join("geocoding"))]
persistentcache::cache_func!(File, dirs::cache_dir().unwrap().join("geocoding"),
pub fn location(address: &str) -> Coordinate {
    use geocoding::Forward;
    Coordinate((*geocoding::openstreetmap::Openstreetmap::new().forward(address).unwrap().first().unwrap_or_else(||panic!("{}",address))).into())
}
);

lazy_static::lazy_static!{
    static ref cache : std::sync::Mutex<static_http_cache::Cache<reqwest::Client>> = {
        let path = std::env::temp_dir().join("static_http_cache");
        std::fs::DirBuilder::new().recursive(true).create(&path).unwrap();
        std::sync::Mutex::new(static_http_cache::Cache::new(path, reqwest::Client::new()).unwrap())
    };
}
#[throws(Box<dyn std::error::Error>)]
pub fn route(coordinates: &[Coordinate]) -> osrm::Response {
    serde_json::from_reader(cache.lock().unwrap().get(Url::parse("http://router.project-osrm.org/route/v1/driving/")?.join(&coordinates.iter().map(|p| p.to_string()).join(";"))?)?)?
}
