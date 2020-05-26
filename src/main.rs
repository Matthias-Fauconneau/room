//use {std::fmt::Debug, anyhow::Context, itertools::Itertools};
/*#[throws] fn process_results<T,E,I:Iterator<Item=Result<T,E>>,R,F:FnOnce(itertools::ProcessResults<&mut I, E>)->R>(mut iter: I, processor: F) -> R
where Result<R,E>:Context<R,E>, Result<T,E>:Debug { itertools::process_results::<_,F,T,E,R>(iter.by_ref(), processor).context(format!("{:?}", iter.format("\n")))? }*/
use itertools::process_results;

use {fehler::throws, chrono::NaiveTime};
#[derive(Debug)] struct Time(NaiveTime); //chrono::DateTime<Local>);
impl<'de> Deserialize<'de> for Time {
    #[throws(D::Error)]
    fn deserialize<D:Deserializer<'de>>(deserializer: D) -> Self {
        Time(NaiveTime::parse_from_str(&String::deserialize(deserializer)?, "%H:%M").map_err(serde::de::Error::custom)?)
    }
}

use {derive_more::Deref, serde::{Deserialize, Deserializer}};

#[derive(Debug,Deserialize,Deref,PartialEq,Eq,PartialOrd,Ord)] struct Room { cost: u16, #[deref] address: String, href: Option<String> }
impl std::fmt::Display for Room { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{:>4} {}", self.cost, self.address)
}}
use url::Url;
impl Room {
    pub fn url(&self, base: &Url) -> Option<Url> { self.href.as_ref().map(|href| base.join(&href).ok()).flatten() }
}

#[derive(Debug,Deserialize)] enum Trip { From(Time), To(Time) }
#[derive(Debug,Deserialize,Deref)] struct Goal { #[deref] address: String, trips: Vec<Trip> }

mod wg;

use anyhow::Error;
#[throws]
fn main() {
    let goals: Vec<Goal> = ron::de::from_reader(std::fs::File::open("../goals.ron")?)?;
    let rooms : Vec<Room> = ron::de::from_reader(std::fs::File::open("../rooms.ron")?)?;
    let rooms = rooms.into_iter().map(Ok).chain(wg::rooms()?.map(|r|r.map(|wg::Room{cost,address,href,..}| Room{cost,address,href:Some(href)})));
    process_results(rooms, |rooms| {
        use itertools::Itertools;
        let rooms : Vec<_> = rooms.sorted().filter(|r| r.address.len()>0).take(3).collect();
        eprintln!("{}", rooms.iter().format("\n"));
        println!("{}", rooms.iter().filter_map(|r| r.url(&wg::host)).format("\n"));
        use newtype::NewType;
        #[derive(NewType)] struct Result<T,E>(std::result::Result<T,E>);
        use std::fmt::Display;
        impl<T:Display,E:Display> Display for Result<T,E> { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match &self.0 { Ok(inner) => Display::fmt(inner, f), Err(inner) => write!(f, "({})", inner) }
        } }
        eprintln!("{}", rooms.iter().map(|room| goals.iter().map(move |goal| map::distance(room, goal)).map(Result::from).format(" ")).format("\n"))
    })?
}
