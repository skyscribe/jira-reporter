extern crate hyper;
extern crate hyper_proxy;
extern crate tokio_core;
extern crate futures;

use self::hyper::{Client, Request, Chunk, Method, Uri};
use self::hyper::client::{HttpConnector};
use self::hyper_proxy::{Proxy, ProxyConnector, Intercept};
use self::tokio_core::reactor::Core;
use self::futures::future::{Future};
use self::futures::Stream;

pub fn fetch(core: &mut Core) -> impl Future {
    //proxied client
    let client = prepare_proxied_client(core);

    //create request
    let input = "http://httpbin.org/ip";
    let uri : Uri = input.parse().expect("Invalid uri provided!");
    let mut req = Request::new(Method::Get, uri.clone());
    //if let Some(headers) = proxy.http_headers(&uri) {
    //    req.headers_mut().extend(headers.iter());
    //}
    req.set_proxy(true);

    client.request(req)
        .and_then(|res| {
            info!("Received response now! {}", res.status());
            res.body().concat2()
        })
        .map(move |body: Chunk| {
            ::std::str::from_utf8(&body).unwrap().to_string()
        })
        .map_err(|err| {
            error!("Request failed by: {}", err);
            err
        })
}

fn prepare_proxied_client(core: & mut Core) -> (Client<ProxyConnector<HttpConnector>>) {
    let handle = core.handle();
    let proxy = {
        let proxy_uri = "http://10.14.1.10:8080".parse().unwrap();
        let mut proxy = Proxy::new(Intercept::All,  proxy_uri);
        let connector = HttpConnector::new(4, &handle);
        ProxyConnector::from_proxy(connector, proxy).unwrap()
    };
    Client::configure().connector(proxy).build(&handle)
}