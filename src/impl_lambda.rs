use std::{
  convert::Infallible,
  future::Future,
  pin::Pin,
  sync::Arc,
};
use netlify_lambda_http::{
  Context,
  Handler,
  Request,
  IntoResponse,
  Response,
};

impl IntoResponse for crate::Response {
  fn into_response(self) -> Response<aws_lambda_events::encodings::Body> {
    let (parts, body) = Response::<String>::from(self).into_parts();
    Response::from_parts(parts, body.into())
  }
}

impl Handler for crate::SharedHandler {
  type Response = crate::Response;
  type Error    = Infallible;
  type Fut      = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

  fn call(&mut self, event: Request, _context: Context) -> Self::Fut {
    let handler = Arc::clone(&self.0);
    Box::pin(async move {
      let res = handler
        .lock().await
        .handle(event).await;
      Ok(res)
    })
  }
}
