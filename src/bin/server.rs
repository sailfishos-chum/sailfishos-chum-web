use std::{
  future::Future,
  net::SocketAddr,
  pin::Pin,
  task::{Context, Poll},
};
use hyper::{Server, service::Service};

use sailfishos_chum_web_lambda::{Handler, SharedHandler};

struct MakeHandler(SharedHandler);

impl<T> Service<T> for MakeHandler {
  type Response = SharedHandler;
  type Error    = hyper::Error;
  type Future   = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, _: T) -> Self::Future {
    let cloned = self.0.clone();
    Box::pin(async move {
      Ok(cloned)
    })
  }
}

async fn shutdown_signal() {
  tokio::signal::ctrl_c()
    .await
    .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let addr = SocketAddr::from(([127, 0, 0, 1], 9999));

  let server = Server::bind(&addr)
    .serve(MakeHandler(Handler::shared()));

  println!("Listening on http://{}", addr);

  let graceful = server.with_graceful_shutdown(shutdown_signal());

  graceful.await?;

  Ok(())
}
