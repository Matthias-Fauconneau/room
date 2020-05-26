mod osrm;
use {fehler::throws, anyhow::{Error, Result, ensure, anyhow}, itertools::Itertools, client::{Url, Client, client}, persistent_cache::cache};
pub use osrm::Coordinate;

cache!{home geocoding,
pub fn location(address: &str) -> Result<Coordinate> {
    ensure!(address.len()>0, "Empty address");
    use ::geocoding::Forward;
    Ok(Coordinate((*::geocoding::openstreetmap::Openstreetmap::new().forward(address).unwrap().first().ok_or_else(||anyhow!("{}",address))?).into()))
}}

trait Route {
    #[throws] fn route(&mut self, coordinates: &[Coordinate]) -> osrm::Response;
}

impl<T:Client> Route for T {
    #[throws] fn route(&mut self, coordinates: &[Coordinate]) -> osrm::Response {
        serde_json::from_slice(&self.get(Url::parse("http://router.project-osrm.org/route/v1/driving/")?.join(&coordinates.iter().map(|p| p.to_string()).join(";"))?)?)?
    }
}

cache!{tmp route, pub fn cached_route(coordinates: &[Coordinate]) -> Result<osrm::Response> { client().route(coordinates) }} // skips If-Modified
pub fn route(coordinates: &[Coordinate]) -> Result<osrm::Response> {
    if cached_route(coordinates).is_err() { use persistent_cache::PersistentCache; route::storage().remove(&persistent_cache::key(&(coordinates,)))?; }
    client().route(coordinates)
}

#[throws] pub fn distance(from: &str, to: &str) -> f32 { route(&[location(from)?, location(to)?])?.routes[0].distance }
