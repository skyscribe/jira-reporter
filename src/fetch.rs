extern crate hyper;
extern crate hyper_proxy;
extern crate tokio_core;
extern crate futures;

use self::hyper::{Client, Request, Chunk, Method, Uri};
use self::hyper::client::{HttpConnector};
use self::hyper::header::{Basic, Accept, qitem, Authorization, UserAgent};
use self::hyper::mime;
use self::hyper_proxy::{Proxy, ProxyConnector, Intercept};
use self::tokio_core::reactor::Core;
use self::futures::future::{Future};
use self::futures::Stream;

pub fn fetch(core: &mut Core, input: &str, login: Basic) -> impl Future {
    //proxied client
    let uri : Uri = input.parse().expect("Invalid uri provided!");
    let mut req = Request::new(Method::Get, uri.clone());
    let client = prepare_proxied_client(core, &mut req, &uri, login);

    client.request(req)
        .and_then(|res| {
            info!("Received response now! {}", res.status());
            res.body().concat2()
        })
        .map(move |body: Chunk| {
            let body_str = ::std::str::from_utf8(&body).unwrap().to_string();
            debug!("Body is: <{}>", &body_str);
            body_str
        })
        .map_err(|err| {
            error!("Request failed by: {}", err);
            err
        })
}

fn prepare_proxied_client(core: & mut Core, req: &mut Request, uri: &Uri, auth: Basic)
        -> Client<ProxyConnector<HttpConnector>> {
    let handle = core.handle();
    let proxy = {
        let proxy_uri = "http://10.144.1.10:8080".parse().unwrap();
        let mut proxy = Proxy::new(Intercept::All,  proxy_uri);
        proxy.set_authorization(auth.clone());

        let connector = HttpConnector::new(4, &handle);
        ProxyConnector::from_proxy(connector, proxy).unwrap()
    };

    if let Some(headers) = proxy.http_headers(&uri) {
        req.headers_mut().extend(headers.iter());
        req.set_proxy(true);
    } else {
        debug!("No headers found for proxy!");
        req.headers_mut().set(UserAgent::new("MyScript"));
        req.headers_mut().set(Accept(vec![qitem(mime::APPLICATION_JSON)]));
        req.headers_mut().set(Authorization(auth));
    }

    //debug headers
    for hdr_item in req.headers().iter() {
        debug!("Header==={}", hdr_item);
    }

    Client::configure().connector(proxy).build(&handle)
}