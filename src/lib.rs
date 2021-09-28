#![feature(io_read_to_string)]
#![feature(try_trait_v2)]

use std::{
  convert::{Infallible, TryFrom},
  fmt::Display,
  io::prelude::*,
  ops::FromResidual,
  sync::Arc,
};
use hyper::{Body, body::Buf, client::HttpConnector};
use hyper_tls::HttpsConnector;
use tokio::sync::Mutex;
use http::{Request, status::StatusCode};
use serde::Serialize;

#[cfg(feature = "lambda")]
pub mod impl_lambda;

#[cfg(feature = "server")]
pub mod impl_server;

#[derive(Serialize)]
struct RpmUrls {
  chum: String,
  gui:  String,
}

struct Repo {
  checksum: String,
  rpm_urls: RpmUrls,
}

#[derive(Serialize)]
pub struct Error {
  error: String,
}

pub struct Response(StatusCode, String);

impl Response {
  fn error<E: Display>(status: StatusCode, reason: E) -> Self {
    Self(status, format!(r#"{{"error": "{}"}}"#, reason))
  }

  fn error500<E: Display>(reason: E) -> Self {
    Self::error(StatusCode::INTERNAL_SERVER_ERROR, reason)
  }
}

impl<D: Serialize> From<(StatusCode, &D)> for Response {
  fn from((status, data): (StatusCode, &D)) -> Self {
    Self(status, serde_json::to_string(data).unwrap())
  }
}

impl From<&RpmUrls> for Response {
  fn from(urls: &RpmUrls) -> Self {
    (StatusCode::OK, urls).into()
  }
}

impl From<&str> for Response {
  fn from(error: &str) -> Self {
    Self::error500(error)
  }
}

impl<E: Display> FromResidual<Result<Infallible, E>> for Response {
  fn from_residual(residual: Result<Infallible, E>) -> Self {
    Self::error500(residual.err().unwrap())
  }
}

impl From<Response> for http::Response<String> {
  fn from(res: Response) -> Self {
    http::Response::builder()
      .status(res.0)
      .header(http::header::CONTENT_TYPE, "application/json")
      .body(res.1)
      .unwrap()
  }
}

struct ClientResponse<B: Buf>(B);

impl<B: Buf> ClientResponse<B> {
  fn reader(self) -> impl Read {
    self.0.reader()
  }

  fn text(self) -> String {
    std::io::read_to_string(&mut self.0.reader()).unwrap()
  }
}

struct Client {
  client:   hyper::Client<HttpsConnector<HttpConnector>, Body>,
  repo_url: String,
}

impl Client {
  fn new(repo_id: &str) -> Self {
    let repo_base = "https://repo.sailfishos.org/obs/sailfishos:/chum/";
    let mut repo_url = String::new();
    repo_url.reserve(repo_base.len() + repo_id.len());
    repo_url += repo_base;
    repo_url += repo_id;

    Self {
      client: hyper::Client::builder().build(HttpsConnector::new()),
      repo_url,
    }
  }

  async fn get(&self, path: &str) -> Result<ClientResponse<impl Buf>, hyper::Error> {
    let url  = format!("{}/{}", self.repo_url, path);
    println!("GET {}", url);
    let uri  = hyper::Uri::try_from(url).unwrap();
    let res  = self.client.get(uri).await?;
    let body = hyper::body::aggregate(res).await?;
    Ok(ClientResponse(body))
  }
}

pub struct Handler {
  repo_cache: std::collections::HashMap<String, Repo>,
}

impl Clone for SharedHandler {
  fn clone(&self) -> Self {
    SharedHandler(Arc::clone(&self.0))
  }
}

impl Handler {
  pub fn shared() -> SharedHandler {
    SharedHandler(Arc::new(Mutex::new(Self {
      repo_cache: std::collections::HashMap::new(),
    })))
  }

  async fn handle<T>(&mut self, req: Request<T>) -> Response<> {
    let parts = req
      .uri()
      // Path without leading `/`
      .path()[1..]
      .split('/')
      .collect::<Vec<&str>>();

    let repo_id = match parts.len() {
      n if n >= 2 => parts[n-2..].join("_"),
      _ => return Response::error(StatusCode::NOT_FOUND, "Not found"),
    };

    let client = Client::new(&repo_id);

    let repomd = client.get("repodata/repomd.xml")
      .await?
      .text();
    let repomd = roxmltree::Document::parse(&repomd)?;

    let primary = match repomd.descendants().find(|e| { e.attribute("type") == Some("primary") }) {
      Some(v) => v,
      None    => return r#"Cannot find "primary" data in repomd.xml"#.into(),
    };

    let checksum = match primary.children().find(|e| e.has_tag_name("checksum")) {
      Some(v) => v.text().unwrap(),
      None    => return r#""primary" data in repomd.xml has no "checksum""#.into(),
    };

    let Repo{rpm_urls, ..} = match self.repo_cache.get(&repo_id) {
      Some(r) if r.checksum == checksum => r,
      _ => {
        let location = match primary.children().find(|e| e.has_tag_name("location")) {
          Some(v) => match v.attribute("href") {
            Some(v) => v,
            None    => return r#""location" tag for "primary" data in repomd.xml has no "href" attribute"#.into(),
          },
          None => return r#""primary" data in repomd.xml has no "location""#.into(),
        };
        let primary = client.get(location).await?;
        let mut primary_reader = flate2::read::GzDecoder::new(primary.reader());
        let mut primary = String::new();
        primary_reader.read_to_string(&mut primary)?;
        let primary  = roxmltree::Document::parse(&primary)?;
        let mut chum = String::new();
        let mut gui  = String::new();
        for pkg in primary.descendants() {
          if pkg.has_tag_name("package") {
            if let Some(name) = pkg.children().find(|e| e.has_tag_name("name")) {
              let get_url = || {
                match pkg.children().find(|e| e.has_tag_name("location") && e.has_attribute("href")) {
                  Some(l) => format!("{}/{}", client.repo_url, l.attribute("href").unwrap()),
                  None    => String::new(),
                }
              };
              match name.text() {
                Some(n) if n == "sailfishos-chum"     => chum = get_url(),
                Some(n) if n == "sailfishos-chum-gui" => gui  = get_url(),
                _ => ()
              }
            }
          }
        }
        let repo = Repo {
          checksum: checksum.into(),
          rpm_urls: RpmUrls {
            chum,
            gui,
          }
        };
        self.repo_cache.insert(repo_id.clone(), repo);
        self.repo_cache.get(&repo_id).unwrap()
      }
    };

    rpm_urls.into()
  }
}

pub struct SharedHandler(Arc<Mutex<Handler>>);
