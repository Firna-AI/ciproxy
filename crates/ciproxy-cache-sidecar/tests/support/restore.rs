//! Public HTTP restore requests shared by cache source regression scenarios.

use serde_json::Value;

use super::job::Job;

pub async fn lookup(job: &Job, legacy: bool, key: &str) -> Value {
    if legacy {
        job.client
            .get(format!(
                "{}/_apis/artifactcache/cache?keys={key},deps-&version=v1",
                job.origin
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    } else {
        job.lookup(key, "v1", &["deps-"]).await
    }
}

pub async fn contents(job: &Job, legacy: bool, key: &str) -> String {
    let entry = lookup(job, legacy, key).await;
    let field = if legacy {
        "archiveLocation"
    } else {
        "signedDownloadUrl"
    };
    assert!(
        entry[field].is_string(),
        "restore returned no archive: {entry}"
    );
    job.client
        .get(entry[field].as_str().unwrap())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}
