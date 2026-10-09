use crate::{Error, Launch, Session};

#[tokio::test]
async fn missing_binary_is_spawn_error() {
    match Session::launch(Launch::new("/no/such/chrome")).await {
        Ok(_) => panic!("spawn should fail"),
        Err(error) => assert!(matches!(error, Error::Spawn { .. }), "{error}"),
    }
}

#[tokio::test]
async fn a_browser_that_exits_at_start_is_reported_at_once() {
    let started = std::time::Instant::now();
    match Session::launch(Launch::new("/usr/bin/false")).await {
        Ok(_) => panic!("launch should fail"),
        Err(error) => assert!(matches!(error, Error::Exited { .. }), "{error}"),
    }
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
}
