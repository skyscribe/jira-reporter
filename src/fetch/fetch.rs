extern crate hyper;
extern crate hyper_proxy;
extern crate tokio_core;
extern crate futures;

use std::rc::Rc;
use self::hyper::{Client, Request, Chunk, Method, Uri, StatusCode, Response};
use self::hyper::client::{HttpConnector};
use self::hyper::header::{Basic, Accept, qitem, Authorization, UserAgent, ContentType};
use self::hyper::{mime, Error};
use self::hyper_proxy::{Proxy, ProxyConnector, Intercept};
use self::tokio_core::reactor::Core;
use self::futures::future::{Future, err, Either, FutureResult};
use self::futures::Stream;

#[derive(Debug)]
pub struct RequestInfo{
    method: Method,
    uri: Uri,
    body: Option<String>,
}

impl RequestInfo {
    pub fn post(uri: &str, body: &str) -> RequestInfo{
        RequestInfo{
            method: Method::Post,
            uri: uri.parse().unwrap(),
            body: Some(body.to_string()), 
        }
    }

    pub fn get(uri: &str) -> RequestInfo {
        RequestInfo {
            method: Method::Get,
            uri: uri.parse().unwrap(),
            body: None,
        }
    }
} 

#[derive(Debug)]
pub struct Fetcher {
    login: Rc<Basic>,
    client: Option<Client<ProxyConnector<HttpConnector>>>,
}

impl Fetcher{
    pub fn new(login: Rc<Basic>) -> Fetcher {
        Fetcher {
            login: login.clone(),
            client: None,
        }
    }

    pub fn query_with<P>(&mut self, req: RequestInfo, core:&mut Core, p: Option<P>) 
            -> impl Future<Item=String, Error=String>
                where P: FnOnce(&str) -> () {
        if self.client.is_none() {
            info!("Creating client connection since not exist yet!");
            self.client = Some(self.prepare_proxied_client(core));
        }

        info!("Request for {}", &req.uri);
        let mut request = Request::new(req.method, req.uri);
        {self.set_request_headers(&mut request)};
        if let Some(body) = req.body {
            request.headers_mut().set(ContentType::json());
            request.set_body(body.clone());
        }

        //perform request now
        self.client.as_ref().unwrap()
            .request(request)
            .and_then(move |res| {
                let b = res.body().concat2();
                match res.status() {
                    StatusCode::Ok => Either::A({
                        res.body().concat2()
                            .map(move |body: Chunk| self.handle_ok(body, p))
                    }),
                    _ => Either::B(self.handle_nok(res)),
                }
            }).map_err(|err|{
                warn!("HTTP error occurred = {}!", err);
                "failure".to_string()
            })
    }

    fn handle_ok<P>(&self, body: Chunk, p: Option<P>) -> String
            where P: FnOnce(&str) -> () {
        trace!("About to parse chunks!");
        let body_str = ::std::str::from_utf8(body).unwrap().to_string();
        trace!("Body is: <{}>", &body_str);
        if let Some(parser) = p {
            parser(&body_str);
        }
        "processed".to_string()
    }

    fn handle_nok(&self, res: Response) -> FutureResult<String, Error> {
        error!("Error code is: {}", res.status());
        err::<String, Error>(Error::Status)
    }

    fn prepare_proxied_client(&self, core: & mut Core) 
            -> Client<ProxyConnector<HttpConnector>> {
        let handle = core.handle();
        let proxy = {
            let proxy_uri = "http://10.144.1.10:8080".parse().unwrap();
            let mut proxy = Proxy::new(Intercept::All,  proxy_uri);
            proxy.set_authorization((*self.login).clone());

            let connector = HttpConnector::new(4, &handle);
            ProxyConnector::from_proxy(connector, proxy).unwrap()
        };
        Client::configure().connector(proxy).build(&handle)
    }

    fn set_request_headers(&self, req: &mut Request) {
        req.set_proxy(true);
        req.headers_mut().set(UserAgent::new("MyScript"));
        req.headers_mut().set(Accept(vec![qitem(mime::APPLICATION_JSON)]));
        req.headers_mut().set(Authorization((*self.login).clone()));

        //debug headers
        for hdr_item in req.headers().iter() {
            debug!("Header==={}", hdr_item);
        }
    }
}