use std::rc::Rc;
use hyper::{Client, Request, Chunk, Method, Uri};
use hyper::client::{HttpConnector};
use hyper::header::{Basic, Accept, qitem, Authorization, UserAgent, ContentType};
use hyper::{mime, StatusCode, Error};
use hyper_proxy::{Proxy, ProxyConnector, Intercept};
use tokio_core::reactor::Core;
use futures::future::{Future, ok};
use futures::Stream;

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
            -> impl Future<Item=String, Error=Error>
                where P: FnOnce(&str, StatusCode) -> () {
        if self.client.is_none() {
            info!("Creating client connection since not exist yet!");
            self.client = Some(self.prepare_proxied_client(core));
        }

        info!("Request for {}", &req.uri);
        let mut request = Request::new(req.method, req.uri);
        self.set_request_headers(&mut request);
        if let Some(body) = req.body {
            request.headers_mut().set(ContentType::json());
            request.set_body(body.clone());
        }

        //perform request now
        self.client.as_ref().unwrap()
            .request(request)
            .and_then(|res| {
                info!("Received response now! {}", res.status());
                let status = ok::<StatusCode, Error>(res.status());
                res.body().concat2().join(status)
            })
            .map(move |result: (Chunk, StatusCode)| {
                use std::str::from_utf8;
                let (body, code) = result;
                let body_str:&str = match code {
                    StatusCode::Ok => from_utf8(&body).unwrap(),
                    _ => "invalid, don't parse",
                };
                
                if let Some(parser) = p {
                    parser(&body_str, code);
                }
                
                "performed".to_string()
            })
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