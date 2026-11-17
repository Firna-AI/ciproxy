//! Orphan relocation cannot expose another selected label after a failure.

use std::sync::Arc;

use ciproxy_github::app::{AppService, GitHubApp};
use ciproxy_github::error::Error;
use ciproxy_github::scale_sets::ScaleSetFactory;
use ciproxy_store_interface::store::Store;

use super::super::support::{Factory, id};
use super::http::Server;

pub(super) async fn failed_relocation_keeps_all_selected_labels_isolated(
    store: Arc<dyn Store>,
    app: &AppService,
    server: &Server,
) {
    let first_label = "ciproxy-a-orphan";
    let later_label = "ciproxy-quarantined-53-1-quarantine";
    let current = store.organization(id(1), id(5)).unwrap().unwrap();
    let api = Factory(server.origin.clone())
        .for_organization(&current)
        .await
        .unwrap();
    let labels = store.labels(id(1)).unwrap();
    let requested_labels: Vec<_> = labels.iter().map(|label| label.name.as_str()).collect();
    api.rename_scale_set(53, first_label, &requested_labels)
        .await
        .unwrap();
    for (label_id, name) in [(406, first_label), (407, later_label)] {
        let mut label = store.labels(id(1)).unwrap().remove(0);
        label.id = id(label_id);
        label.name = name.to_owned();
        store.create_label(id(1), &label).unwrap();
    }
    server.state.lock().unwrap().fail_create = Some(first_label.to_owned());
    assert!(matches!(
        app.refresh_repositories(8).await,
        Err(Error::Internal(_))
    ));
    assert_eq!(
        server.eligible_runners(later_label),
        Vec::<i64>::new(),
        "a failed earlier ensure must not expose another selected label"
    );
    assert_eq!(server.state.lock().unwrap().runners.len(), 3);
    app.refresh_repositories(8).await.unwrap();
    let bindings = store.scale_sets(id(1), id(5)).unwrap();
    for label_id in [406, 407] {
        let binding = bindings
            .iter()
            .find(|set| set.label_id == id(label_id))
            .unwrap();
        assert_ne!(binding.github_id, 53);
    }
    assert_eq!(server.eligible_runners(later_label), Vec::<i64>::new());
    assert_eq!(
        server.state.lock().unwrap().creates,
        vec![83, 84, 85, 86, 87, 88, 89]
    );
    assert_eq!(server.state.lock().unwrap().runners.len(), 3);
    assert_eq!(server.state.lock().unwrap().deletes, Vec::<i64>::new());
    let mut label = store.labels(id(1)).unwrap().remove(0);
    label.id = id(408);
    label.name = "ciproxy-a-quarantine".to_owned();
    store.create_label(id(1), &label).unwrap();
    app.refresh_repositories(8).await.unwrap();
    let safe = store
        .scale_sets(id(1), id(5))
        .unwrap()
        .into_iter()
        .find(|set| set.label_id == label.id)
        .unwrap();
    {
        let mut remote = server.state.lock().unwrap();
        let quarantined = remote.sets.get_mut(&53).unwrap();
        quarantined.name = "ciproxy-a".to_owned();
        quarantined.labels = vec![label.name.clone()];
        let retained = remote.sets.get_mut(&safe.github_id).unwrap();
        retained.name = "safe-retired".to_owned();
        retained.labels = vec!["safe-retired".to_owned()];
    }
    app.refresh_repositories(8).await.unwrap();
    assert_eq!(
        server.eligible_runners(&label.name),
        Vec::<i64>::new(),
        "restoring a safe retained set must release an orphan marker first"
    );
    assert_eq!(
        store
            .scale_sets(id(1), id(5))
            .unwrap()
            .into_iter()
            .find(|set| set.label_id == label.id)
            .unwrap(),
        safe
    );
    assert_eq!(
        server.state.lock().unwrap().sets[&safe.github_id].labels,
        vec![label.name.clone()]
    );
    assert_eq!(
        server.state.lock().unwrap().creates,
        vec![83, 84, 85, 86, 87, 88, 89, 90]
    );
}
