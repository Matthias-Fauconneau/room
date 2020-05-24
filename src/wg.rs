#![allow(non_camel_case_types,non_snake_case)]
use {serde::Serialize, smart_default::SmartDefault, bytes::Bytes};

#[derive(Serialize,SmartDefault,Hash)] enum Permanent { #[default] all }
#[derive(Serialize,SmartDefault,Hash)] enum Student { #[default] none }
#[derive(Serialize,SmartDefault,Hash)] enum OrderBy { #[serde(rename="@sortDate")]#[default] sortDate }
#[derive(Serialize,SmartDefault,Hash)] enum OderDirection { #[default] descending }
#[derive(Serialize,SmartDefault,Hash)]
struct Search {
    query: String,
    priceMin: u32,
    #[default = 1500] priceMax: u32,
    state: String,
    permanent: Permanent,
    student: Student,
    orderBy: OrderBy,
    orderDir: OderDirection,
    #[default = true] startSearchMate: bool,
    #[default = true] wgStartSearch: bool,
    start: u32,
}

use {anyhow::{Error, bail}, fehler::throws};

#[test] #[throws] fn test(){
    assert_eq!(serde_urlencoded::to_string(Search{state: "zurich-stadt".to_string(), ..Default::default()})?,
    "query=&priceMin=0&priceMax=1500&state=zurich-stadt&permanent=all&student=none&orderBy=%40sortDate&orderDir=descending&startSearchMate=true&wgStartSearch=true&start=0");
}

use reqwest::Url;

persistentcache::cache_func!(File, dirs::cache_dir().unwrap().join("post"), // Cache HTTP post for development
    pub fn post(url: Url, form: &(impl Serialize+std::hash::Hash)) -> String { reqwest::blocking::Client::new().post(url).form(form).send().unwrap().text().unwrap() }
);

#[throws]
pub fn search() -> Bytes {
    let html = post(Url::parse("https://www.wgzimmer.ch/en/wgzimmer/search/mate.html")?, &Search{state: "zurich-stadt".to_string(), ..Default::default()});
    use kuchiki::traits::TendrilSink/*one*/;
    let document = kuchiki::parse_html().one(html);
    use itertools::Itertools;
    let a = document.select("html body #main #container #content ul li a:nth-of-type(2)").unwrap();
    pub trait Ok<T, E> { fn ok(self) -> Result<T, E>; }
    //impl<T, E:Default> Ok<T, E> for Option<T> { fn ok(self) -> Result<T, E> { self.ok_or(Default::default()) } }
    impl<T> Ok<T, std::fmt::Error> for Option<T> { fn ok(self) -> Result<T, std::fmt::Error> { self.ok_or(Default::default()) } }
    bail!("{}", a.format_with("\n", |a,f| f( &a.as_node().as_element().ok()?.attributes.borrow().get("href").ok()? ) ) );
    //bail!("{}", a.format_with("\n", |a,f|->core::fmt::Result { f( &a.as_node().as_element().ok()?.attributes.borrow().get("href").ok()? ) } ) );
}
