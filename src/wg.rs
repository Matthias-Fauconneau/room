#![allow(non_camel_case_types,non_snake_case,non_upper_case_globals)]
use {persistent_cache::cache, anyhow::Result, reqwest::Url};

cache!{tmp wget, pub fn get(url: Url) -> Result<Vec<u8>> { use client::{Client, client}; client().get(url) } }
#[throws] pub fn get_text(url: Url) -> String { String::from_utf8(get(url)?)? }
cache!{home wg, fn post(url: Url, form: &(impl Serialize+std::hash::Hash)) -> Result<String> { Ok(reqwest::blocking::Client::new().post(url).form(form).send()?.text()?) } }

use newtype::NewType;
#[derive(NewType)] struct NodeDataRef<T>(kuchiki::NodeDataRef<T>);

impl<T:AsRef<str>> Extend<NodeDataRef<std::cell::RefCell<T>>> for String {
    fn extend<I:IntoIterator<Item=NodeDataRef<std::cell::RefCell<T>>>>(&mut self, iter: I) { iter.into_iter().for_each(move |s| self.push_str(s.borrow().as_ref())) }
}

impl<T:AsRef<str>> std::iter::FromIterator<NodeDataRef<std::cell::RefCell<T>>> for String {
    fn from_iter<I:IntoIterator<Item=NodeDataRef<std::cell::RefCell<T>>>>(iter: I) -> Self { let mut c = Self::new(); c.extend(iter); c }
}

trait Text { fn text(&self) -> String; }
impl Text for kuchiki::NodeDataRef<kuchiki::ElementData> {
    fn text(&self) -> String {
        use kuchiki::iter::NodeIterator;
        self.as_node().children().text_nodes().map(NodeDataRef::from).collect::<String>()
    }
}

use error::{OkOr, Ok};

pub trait Get { #[throws] fn get(&self, selectors: &'static str) -> kuchiki::NodeDataRef<kuchiki::ElementData>; }
impl Get for kuchiki::NodeRef { #[throws] fn get(&self, selectors: &'static str) -> kuchiki::NodeDataRef<kuchiki::ElementData> {
    self.select_first(selectors).ok_or("selector").context(format!("{} {}",selectors, self.to_string()))?
} }

use nom::{IResult, combinator::all_consuming, error::{VerboseError, convert_error}};
#[throws] fn parse<'t,O>(parser: impl Fn(&'t str) -> IResult<&'t str, O, VerboseError<&'t str>>, input: &'t str) -> O {
    all_consuming(parser)(input).map_err(|e| e.map(|e|anyhow!("{}",convert_error(input, e))))?.1
}

#[derive(NewType,Debug,PartialEq,Eq,PartialOrd,Ord)] pub struct Date(chrono::NaiveDate);
use newtype_derive::*; NewtypeDisplay! { () struct Date(chrono::NaiveDate); }

use anyhow::Context;

impl Date {
    #[throws] fn parse_from_str(s: &str) -> Self { Date(chrono::NaiveDate::parse_from_str(s, "%d.%m.%Y ").context(format!("'{}'",s))?) }
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord)] pub struct Room {
    pub cost: u16,
    pub create_date: Date,
    pub from_date: Date,
    pub until: Option<String>,
    pub href: String,
    pub address: String,
}
impl std::fmt::Display for Room { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{} {:>4}F {}{}",self.create_date.format("%d.%m"), self.cost, self.from_date.format("%d.%m.%y"), self.until.as_deref().map(|s|format!("-{}",s)).unwrap_or_default())
}}

use {serde::Serialize, smart_default::SmartDefault};

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

use {anyhow::{Error, anyhow}, fehler::throws};

#[test] #[throws] fn test(){
    assert_eq!(serde_urlencoded::to_string(Search{state: "zurich-stadt".to_string(), ..Default::default()})?,
    "query=&priceMin=0&priceMax=1500&state=zurich-stadt&permanent=all&student=none&orderBy=%40sortDate&orderDir=descending&startSearchMate=true&wgStartSearch=true&start=0");
}

lazy_static::lazy_static! { pub static ref host : Url = Url::parse("https://www.wgzimmer.ch").unwrap(); }
/*impl Room {
    pub fn url(&self) -> Url { host.join(&self.href).unwrap() }
}*/

#[throws]
pub fn rooms() -> impl Iterator<Item=Result<Room>> {
    fn document(html: &str) -> kuchiki::NodeRef {
        use kuchiki::traits::TendrilSink/*one*/;
        kuchiki::parse_html().one(html)
    };
    document(&post(host.join("/en/wgzimmer/search/mate.html")?, &Search{state: "zurich-stadt".to_string(), ..Default::default()})?)
    .select("html body #main #container #content ul li a:nth-of-type(2)").ok_or("selector")?.map(|a| {
        let a = a.as_node();
        let href = a.as_element().ok()?.attributes.borrow().get("href").ok()? .to_owned();
        use nom::{combinator::{opt, map, map_res}, sequence::{pair, preceded, terminated, delimited}, bytes::complete::tag, character::complete::{char, digit1}};
        pub fn integer<'t, E:nom::error::ParseError<&'t str>>(input: &'t str) -> IResult<&'t str,u16, E> { map_res(digit1, |s:&'t str| s.parse::<u16>())(input) }
        let cost = delimited(tag("SFr. "), map(pair(opt(terminated(integer,char('\''))),integer), |(k,u)| k.unwrap_or(0)*1_000+u), tag(".00"));
        let room = document(&get_text(host.join(&href)?)?);
        Ok(Room{
            href,
            create_date: Date::parse_from_str(&a.get("span.create-date strong")?.text_contents()).context(a.to_string())?,
            from_date: Date::parse_from_str(&a.get("span.from-date strong")?.text_contents())?,
            until: parse(
                preceded(tag("  Until: "), |i:&str| Ok(("", Some(i.trim_end()).filter(|s|s!=&"No time restrictions")))),
                &a.get("span.from-date")?.text()
            )?.map(ToOwned::to_owned),
            cost: parse(cost, &a.get("span.cost strong")?.text_contents() )?,
            address: room.get("html body #main #container #content .result .adress-region p:nth-child(3)")?.text(),
        })
    })
}
