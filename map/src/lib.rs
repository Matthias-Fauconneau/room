mod osrm;
use {fehler::throws, anyhow::Error, itertools::Itertools, client::{Url, Client, client}};
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

persistentcache::cache_func!(File, std::env::temp_dir().join("route"), // skips If-Modified
pub fn route(coordinates: &[Coordinate]) -> Result<osrm::Response, String> {
    use error::ErrInto;
    client().route(coordinates).err_into()
}
);

#[throws] pub fn distance(from: &str, to: &str) -> f32 {
    use error::error::MapErrToError;
    route(&[location(from).map_err_to_error()?, location(to).map_err_to_error()?]).map_err_to_error()?.routes[0].distance
    //use error::error::From;
    //From::from(route(&[location(from).into()?, location(to).into()?]))?.routes[0].distance
}
