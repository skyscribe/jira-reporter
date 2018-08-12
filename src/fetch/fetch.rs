extern crate hyper;
extern crate hyper_proxy;
extern crate tokio_core;
extern crate futures;

use self::hyper::{Client, Request, Chunk, Method, Uri};
use self::hyper::client::{HttpConnector};
use self::hyper::header::{Basic, Accept, qitem, Authorization, UserAgent, ContentType};
use self::hyper::mime;
use self::hyper_proxy::{Proxy, ProxyConnector, Intercept};
use self::tokio_core::reactor::Core;
use self::futures::future::{Future};
use self::futures::Stream;

#[derive(Debug)]
pub enum FetchMethod {
    Get,
    Post,
}

#[derive(Debug)]
pub struct Fetcher<'a> {
    login: &'a Basic,
    uri: String,
    body: Option<String>,
    method: FetchMethod,
}

impl <'a> Fetcher<'a> {
    pub fn new(input: String, login:&'a Basic) -> Fetcher {
        Fetcher {
            login: &login,
            uri: input.to_string(),
            body: None,
            method: FetchMethod::Get,
        }
    }

    pub fn set_method(&mut self, method: FetchMethod) {
        self.method = method;
    }

    pub fn set_body(&mut self, body: String)  {
        self.body = Some(body);
    }

    pub fn fetch(&self, core: &mut Core) -> impl Future {
        //proxied client
        let uri : Uri = self.uri.parse().expect("Invalid uri provided!");
        let method = match self.method {
            FetchMethod::Get => Method::Get,
            FetchMethod::Post => Method::Post,
        };

        let mut req = Request::new(method, uri.clone());
        if let Some(ref body) = self.body {
            req.headers_mut().set(ContentType::json());
            req.set_body(body.clone());
        }
        let client = self.prepare_proxied_client(core, &mut req, &uri);

        info!("Start to sending request for {}", uri);
        client.request(req)
            .and_then(|res| {
                info!("Received response now! {}", res.status());
                res.body().concat2()
            })
            .map(move |body: Chunk| {
                let body_str = ::std::str::from_utf8(&body).unwrap().to_string();
                info!("Body is: <{}>", &body_str);
                body_str
            })
            .map_err(|err| {
                error!("Request failed by: {}", err);
                err
            })
    }

    fn prepare_proxied_client(&self, core: & mut Core, req: &mut Request, uri: &Uri)
            -> Client<ProxyConnector<HttpConnector>> {
        let handle = core.handle();
        let proxy = {
            let proxy_uri = "http://10.144.1.10:8080".parse().unwrap();
            let mut proxy = Proxy::new(Intercept::All,  proxy_uri);
            proxy.set_authorization(self.login.clone());

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
            req.headers_mut().set(Authorization(self.login.clone()));
        }

        //debug headers
        for hdr_item in req.headers().iter() {
            debug!("Header==={}", hdr_item);
        }

        Client::configure().connector(proxy).build(&handle)
    }
}