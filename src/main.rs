extern crate hyper;

use hyper::{Client, Server, Request, Response, Body};
use anyhow::*;
use std::net::SocketAddr;
use hyper::service::{make_service_fn, service_fn};
use std::sync::{Arc, RwLock};

const TARGET_HOST:&str = "https://www.snoyman.com";

fn mutate_request(req: &mut Request<Body>) -> Result<()>{
    for key in &["content-length","transfer-encoding", "accept-encoding", "content-encoding" ]{
        req.headers_mut().remove(*key);
    }

    let uri = req.uri();
    let uri_string = match uri.query(){
        None => format!("{}{}", TARGET_HOST,  uri.path()),
        Some(query) => format!("{}{}?{}", TARGET_HOST, uri.path(), query),
    };
    *req.uri_mut() = uri_string.parse().context("Parsing URI in mutate request")?;
    Ok(())
}

#[derive(Debug)]
struct Stats{
    proxied: usize,
}

#[tokio::main]
async fn main() -> Result<()>{
    let https = hyper_rustls::HttpsConnector::with_native_roots();
    let client: Client<_, hyper::Body> = Client::builder().build(https);
    let client = Arc::new(client);
    let stats: Arc<RwLock<Stats>> = Arc::new(RwLock::new(Stats{
        proxied: 0,
    }));

    // the outer lambda does not need a 'move' keyword because variable 'make_svc' will not outlive 'client'
    let make_svc = make_service_fn(|_| {
        // rust does not allow moving of lambda-captured variables, so we need to clone them
        // and thereby having local variables for moving( by the async block ).
        let client = client.clone();
        let stats = stats.clone();
        // async blocks are essentially lambdas run in an other thread, which will outlive it's wrapping function
        //  (and variables defined in it), so variables it uses must be moved into it rather than referred
        async move {
            let client = client.clone();
            let stats = stats.clone();
            // the inner lambda NEED a 'move' keyword because this lambda is wrapped into a Future<Result<...>>
            // and returned out of it's defining async block where the 'client' and 'stats' are also defined.
            // so this lambda will outlive variable 'client' and 'stats'.
            Ok(service_fn(move |mut req| {
                let client = client.clone();
                let stats = stats.clone();
                // inner async block also need a move keyword, for the same reason that the outer async block needs one.
                async move {
                    if req.uri().path() == "/status" {
                        let body: Body = format!("{:?}", stats.read().unwrap()).into();
                        Ok(Response::new(body))
                    } else {
                        println!("Proxied: {}", req.uri().path());
                        stats.write().unwrap().proxied += 1;
                        mutate_request(&mut req)?;
                        client.request(req).await.context("Making request to backend server.")
                    }
                }
            }))
        }
    });

    let addr = SocketAddr::from(([0,0,0,0], 3000));
    Server::bind(&addr).serve(make_svc).await.context("Running server")?;

    Ok(())
}
