#![allow(non_upper_case_globals)]

trait Erf { type Output; fn erf(self) -> Self::Output; }
impl<T,E:std::string::ToString> Erf for Result<T,E> {
    type Output = Result<T,String>;
    fn erf(self) -> Self::Output { self.map_err(|e| std::string::ToString::to_string(&e)) }
}

trait Meh { type Output; fn meh(self) -> Self::Output; }
impl<T> Meh for Result<T,String> {
    type Output = Result<T,Error>;
    fn meh(self) -> Self::Output { self.map_err(Error::msg) }
}

mod cache; mod osrm;
use {fehler::throws, anyhow::Error, cache::Client, itertools::Itertools, reqwest::Url};
pub use osrm::Coordinate;

persistentcache::cache_func!(File, dirs::cache_dir().unwrap().join("geocoding"),
pub fn location(address: &str) -> Result<Coordinate, String> {
    if address.len()==0 { return Err("Empty address".to_owned()) }
    use geocoding::Forward;
    Ok(Coordinate((*geocoding::openstreetmap::Openstreetmap::new().forward(address).unwrap().first().ok_or_else(||address.to_owned())?).into()))
}
);

trait Route {
    #[throws] fn route(&mut self, coordinates: &[Coordinate]) -> osrm::Response;
}

impl<T:Client> Route for T {
    #[throws] fn route(&mut self, coordinates: &[Coordinate]) -> osrm::Response {
        serde_json::from_slice(&self.get(Url::parse("http://router.project-osrm.org/route/v1/driving/")?.join(&coordinates.iter().map(|p| p.to_string()).join(";"))?)?)?
    }
}

persistentcache::cache_func!(File, std::env::temp_dir().join("route"), // HTTP is already cached but this also skips the stale request
pub fn route(coordinates: &[Coordinate]) -> Result<osrm::Response, String> { Ok(cache::client().route(coordinates).erf()?) }
);

#[throws] pub fn distance(from: &str, to: &str) -> f32 { route(&[location(from).meh()?, location(to).meh()?]).meh()?.routes[0].distance }
