use std::{
  future::Future,
  hash::{Hash, Hasher},
  pin::Pin,
  sync::Arc,
  task::{Context, Poll},
};
use hyper::{self, Body, Request, Response, service::Service};
use http::{header::*, StatusCode};

impl From<crate::Response> for Response<Body> {
  fn from(res: crate::Response) -> Self {
    let (parts, body) = Response::<String>::from(res).into_parts();
    Response::from_parts(parts, body.into())
  }
}

fn prev_etag<'r>(req: &'r Request<Body>) -> &'r str {
  if let Some(none_match) = req.headers().get(IF_NONE_MATCH) {
    if let Ok(none_match) = none_match.to_str() {
      return none_match
    }
  }
  ""
}

fn content_type(path: &std::path::Path) -> &'static str {
  match path.extension().unwrap().to_str().unwrap() {
    "html" => "text/html",
    "js"   => "text/javascript",
    "css"  => "text/css",
    _      => "text/plain",
  }
}

impl Service<Request<Body>> for crate::SharedHandler {
  type Response = Response<Body>;
  type Error    = hyper::Error;
  type Future   = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
  }

  fn call(&mut self, req: Request<Body>) -> Self::Future {
    let path = req.uri().path();
    if path.starts_with("/.netlify/functions/lambda") {
      let handler = Arc::clone(&self.0);
      Box::pin(async move {
        let res = handler
          .lock().await
          .handle(req).await
          .into();
        Ok(res)
      })
    } else if path == "/" {
      let res = Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header(LOCATION, "/index.html")
        .body(Body::empty())
        .unwrap();
      Box::pin(async move { Ok(res) })
    } else {
      let path = std::env::current_dir().unwrap()
        .join("public")
        .join(&path[1..]);
      let res = if path.is_file() {
        let body = std::fs::read(&path).unwrap();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        body.hash(&mut hasher);
        let etag = hasher.finish().to_string();
        if prev_etag(&req) == etag {
          Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .unwrap()
        } else {
          Response::builder()
            .header(CONTENT_TYPE, content_type(&path))
            .header(ETAG, format!("{}", etag))
            .header(X_CONTENT_TYPE_OPTIONS, "nosniff")
            .body(body.into())
            .unwrap()
        }
      } else {
        Response::builder()
          .status(StatusCode::NOT_FOUND)
          .body(Body::empty())
          .unwrap()
      };
      Box::pin(async move { Ok(res) })
    }
  }
}
