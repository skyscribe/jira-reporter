use futures::future::{ok, Future};
use futures::Stream;

use hyper::client::HttpConnector;
use hyper::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use hyper::{Body, Chunk, Client, Method, Request, Uri};
use hyper::{Error, StatusCode};
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use std::rc::Rc;
use tokio_core::reactor::Core;
use typed_headers::Credentials;

#[derive(Debug)]
pub struct RequestInfo {
    method: Method,
    uri: Uri,
    body: String,
}

impl RequestInfo {
    pub fn post(uri: &str, body: &str) -> RequestInfo {
        RequestInfo {
            method: Method::POST,
            uri: uri.parse().unwrap(),
            body: body.to_string(),
        }
    }

    pub fn get(uri: &str) -> RequestInfo {
        RequestInfo {
            method: Method::GET,
            uri: uri.parse().unwrap(),
            body: String::from("{}"),
        }
    }
}

#[derive(Debug)]
pub struct Fetcher {
    login: Rc<Credentials>,
    client: Option<Client<ProxyConnector<HttpConnector>>>,
}

impl Fetcher {
    pub fn new(login: Rc<Credentials>) -> Fetcher {
        Fetcher {
            login: login.clone(),
            client: None,
        }
    }

    //perfor a single qiuery with given request information and parser function
    pub fn query_with<P>(
        &mut self,
        req: RequestInfo,
        mut core: &mut Core,
        result_parser: Option<P>,
    ) -> impl Future<Item = String, Error = Error>
    where
        P: FnOnce(&str, StatusCode) -> (),
    {
        self.prepare_proxied_client_if_not_created(&mut core);
        let request = self.build_request_and_add_headers(req);

        //perform request now
        self.client
            .as_ref()
            .unwrap()
            .request(request)
            .and_then(|res| {
                info!("Received response now! {}", res.status());
                let status = ok::<StatusCode, Error>(res.status());
                res.into_body().concat2().join(status)
            })
            .map(move |result: (Chunk, StatusCode)| {
                use std::str::from_utf8;
                let (body, code) = result;
                let body_str: &str = match code {
                    StatusCode::OK => from_utf8(&body).unwrap(),
                    _ => "invalid, don't parse",
                };

                if let Some(parser) = result_parser {
                    parser(&body_str, code);
                }

                "performed".to_string()
            })
    }

    fn prepare_proxied_client_if_not_created(&mut self, _core: &mut Core) {
        if self.client.is_some() {
            return
        }

        info!("Creating client connection since not exist yet!");
        //let handle = core.handle();
        let proxy = {
            let proxy_uri = "http://10.144.1.10:8080".parse().unwrap();
            let mut proxy = Proxy::new(Intercept::All, proxy_uri);
            proxy.set_authorization((*self.login).clone());

            let connector = HttpConnector::new(4);
            ProxyConnector::from_proxy(connector, proxy).unwrap()
        };
        self.client = Some(Client::builder().build(proxy))
    }

    fn build_request_and_add_headers(&self, req: RequestInfo) -> Request<Body> {
        info!("Request for {}", &req.uri);

        let mut builder = Request::builder();
        builder.uri(req.uri).method(req.method);

        builder.header(USER_AGENT, "MyScript");
        builder.header(ACCEPT, "application/json");
        builder.header(AUTHORIZATION, format!("{}", (*self.login)));
        builder.header(CONTENT_TYPE, "application/json");

        let request = builder.body(Body::from(req.body)).unwrap();

        //check headers inside if we are correct?
        for hdr in request.headers() {
            debug!("=== header: {:?}", hdr);
        }

        request
    }
}
