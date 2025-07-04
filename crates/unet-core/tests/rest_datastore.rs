use httpmock::prelude::*;
use unet_core::datastore::{DataStore, rest::RestDataStore};

#[tokio::test]
async fn health_check_success() {
    let server = MockServer::start_async().await;
    let m = server
        .mock_async(|when, then| {
            when.method(GET).path("/health");
            then.status(200);
        })
        .await;

    let store = RestDataStore::new(&server.url(""));
    store.health_check().await.expect("health check");
    m.assert_hits_async(1).await;
}
