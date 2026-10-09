use crate::{Error, Launch, Session};

#[tokio::test]
async fn missing_binary_is_spawn_error() {
    match Session::launch(Launch::new("/no/such/chrome")).await {
        Ok(_) => panic!("spawn should fail"),
        Err(error) => assert!(matches!(error, Error::Spawn { .. }), "{error}"),
    }
}
