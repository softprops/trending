extern crate select;
use select::document::Document;
use select::predicate::{Predicate, Class, Name};
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use futures::{Future as StdFuture, Stream as StdStream};
use hyper::Client as HyperClient;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();
    let connector = HttpsConnector::new(4, &core.handle()).unwrap();
    let hyper = HyperClient::configure()
        .connector(connector)
        .keep_alive(true)
        .build(&core.handle());
    let work = hyper
        .get("https://github.com/trending/rust".parse().unwrap())
        .and_then(|resp| {
            resp.body().concat2().map(|bytes| {
                let body = String::from_utf8_lossy(&bytes);
                Document::from(body.as_ref())
                    .find(Name("ol").and(Class("repo-list")).descendant(Name("li")))
                    .map(|node| {
                        let project = node.find(Name("h3").descendant(Name("a")))
                            .next()
                            .unwrap()
                            .text()
                            .trim()
                            .to_owned();
                        let mut parts = project.splitn(2, " / ");
                        let owner = parts.next().unwrap().to_owned();
                        let repo = parts.next().unwrap().to_owned();
                        let desc = node.find(Class("py-1").descendant(Name("p")))
                            .next()
                            .unwrap()
                            .text()
                            .trim()
                            .to_owned();
                        let stars = node.find(Name("div").and(Class("f6")).descendant(Name("a")))
                            .next()
                            .unwrap()
                            .text()
                            .trim()
                            .to_owned();
                        (owner, repo, desc, stars)
                    })
                    .into_iter()
                    .collect::<Vec<_>>()
            })
        });
    println!("{:#?}", core.run(work));
}
