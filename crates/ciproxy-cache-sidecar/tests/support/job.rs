//! The runner's literal JSON and transfer calls through the public proxy.

use ciproxy_protocol::jobs::JobScope;
use ciproxy_protocol::secret::Secret;
use reqwest::Client;
use serde_json::{Value, json};
use tokio::task::JoinHandle;

use super::V2;

pub struct Job {
    pub origin: String,
    pub client: Client,
    pub token: Secret,
    pub github_token: Secret,
    pub scope: JobScope,
    pub task: JoinHandle<()>,
}

impl Job {
    pub async fn rpc(&self, method: &str, body: Value) -> Value {
        let response = self
            .client
            .post(format!("{}{V2}{method}", self.origin))
            .bearer_auth("guest-cannot-choose-upstream-token")
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "{method}");
        response.json().await.unwrap()
    }

    pub async fn lookup(&self, key: &str, version: &str, restore_keys: &[&str]) -> Value {
        self.rpc(
            "GetCacheEntryDownloadURL",
            json!({"key":key,"version":version,"restoreKeys":restore_keys,
            "metadata":{"scope":"refs/heads/main","repository":"other/repository"}}),
        )
        .await
    }

    pub async fn reserve(&self, key: &str, version: &str) -> String {
        self.rpc("CreateCacheEntry", json!({"key":key,"version":version}))
            .await["signedUploadUrl"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    pub async fn finalize(&self, key: &str, version: &str, size: u64) {
        let response = self
            .rpc(
                "FinalizeCacheEntryUpload",
                json!({"key":key,"version":version,"sizeBytes":size.to_string()}),
            )
            .await;
        assert_eq!(response, json!({"ok":true,"entryId":"100"}));
    }

    pub async fn save(&self, key: &str, version: &str, data: &str) {
        let url = self.reserve(key, version).await;
        assert_eq!(
            self.client
                .put(url)
                .header("x-ms-blob-type", "BlockBlob")
                .body(data.to_owned())
                .send()
                .await
                .unwrap()
                .status(),
            201
        );
        self.finalize(key, version, data.len() as u64).await;
    }

    pub async fn contents(&self, key: &str, version: &str, restore_keys: &[&str]) -> String {
        let response = self.lookup(key, version, restore_keys).await;
        assert_eq!(response["ok"], true, "lookup {key}");
        self.client
            .get(response["signedDownloadUrl"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.task.abort();
    }
}
