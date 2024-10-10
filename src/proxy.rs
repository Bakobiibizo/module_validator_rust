use std::net::SocketAddr;
use std::convert::Infallible;
use hyper::{Body, Client, Request, Response, Server, Uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::client::HttpConnector;

pub struct Proxy {
    ip: String,
    port: u16,
    target_base_url: String,
}

impl Proxy {
    pub fn new(ip: String, port: u16, target_base_url: String) -> Self {
        Self { ip, port, target_base_url }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("{}:{}", self.ip, self.port).parse()?;

        println!("Proxy listening on {}", addr);
        println!("Forwarding requests to {}", self.target_base_url);

        let client = Client::new();
        let target_base_url = self.target_base_url.clone();

        let make_svc = make_service_fn(move |_conn| {
            let client = client.clone();
            let target_base_url = target_base_url.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    proxy_service(client.clone(), target_base_url.clone(), req)
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }

        Ok(())
    }
}

async fn proxy_service(
    client: Client<HttpConnector>,
    target_base_url: String,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let path_and_query = req.uri().path_and_query()
        .map(|x| x.to_string())
        .unwrap_or_default();

    let new_uri = format!("{}{}", target_base_url, path_and_query);
    let new_uri: Uri = new_uri.parse().expect("Failed to parse URI");

    // Extract headers before moving req
    let headers = req.headers().clone();

    let (parts, body) = req.into_parts();

    let mut new_req = Request::builder()
        .method(parts.method)
        .uri(new_uri)
        .body(body)
        .expect("Failed to build request");

    *new_req.headers_mut() = headers;

    client.request(new_req).await
}