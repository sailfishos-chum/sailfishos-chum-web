use netlify_lambda_http::{
  handler,
  lambda,
};

use sailfishos_chum_web_lambda::Handler;

#[tokio::main]
async fn main() {
  let chum_handler = Handler::shared();
  lambda::run(handler(chum_handler)).await.unwrap();
}
