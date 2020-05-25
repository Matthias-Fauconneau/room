#![allow(non_upper_case_globals)]
mod cache; mod osrm;
use {fehler::throws, anyhow::Error, cache::Client, itertools::Itertools, reqwest::Url};
pub use osrm::Coordinate;

//#[persistentcache::persistent_cache]#[params(FileStorage, dirs::cache_dir().join("geocoding"))]
persistentcache::cache_func!(File, dirs::cache_dir().unwrap().join("geocoding"),
pub fn location(address: &str) -> Result<Coordinate,String> {
    if address.len()==0 { return Err("Empty address".to_owned()) }
    use geocoding::Forward;
    Ok(Coordinate((*geocoding::openstreetmap::Openstreetmap::new().forward(address).unwrap().first().ok_or_else(||address.to_owned())?).into()))
}
);

trait Route {
    #[throws] fn route(&self, coordinates: &[Coordinate]) -> osrm::Response;
}

impl<T:Client> Route for T {
    #[throws] fn route(&self, coordinates: &[Coordinate]) -> osrm::Response {
        serde_json::from_slice(&self.get(Url::parse("http://router.project-osrm.org/route/v1/driving/")?.join(&coordinates.iter().map(|p| p.to_string()).join(";"))?)?)?
    }
}

persistentcache::cache_func!(File, dirs::cache_dir().unwrap().join("route"), // HTTP is already cached but this also skips the stale request
pub fn route(coordinates: &[Coordinate]) -> osrm::Response { cache::client().route(coordinates).unwrap() }
);

pub fn distance(from: &str, to: &str) -> Option<f32> { Some(route(&[location(from).ok()?, location(to).ok()?]).routes[0].distance) }
