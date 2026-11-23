//! Label queries and private resource row mappings.

use ciproxy_protocol::platform::Platform;
use ciproxy_schema::labels::labels;
use ciproxy_store_interface::error::{Error, Result, ResultExt};
use ciproxy_store_interface::labels::{Label, LabelPlace, LabelStore};
use ciproxy_store_interface::vm_size::VmSize;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};
use crate::gateway_transaction::{read_snapshot, transaction};
use crate::label_places;
use crate::overflow_provider_settings::offered_providers;
use crate::platform_text::{parse_platform, platform_text};
use crate::workspaces::lock_workspace;

#[derive(Queryable, Selectable)]
#[diesel(table_name = labels)]
pub(crate) struct LabelRow {
    id: Uuid,
    name: String,
    platform: String,
    vcpus: i32,
    memory_gb: i32,
    disk_gb: i32,
    when_nothing_fits: String,
}

impl LabelRow {
    pub(crate) fn into_record(self, order: Vec<LabelPlace>) -> Result<Label> {
        Ok(Label {
            id: self.id,
            name: self.name,
            platform: parse_platform(&self.platform)?,
            size: VmSize {
                vcpus: self.vcpus,
                memory_gb: self.memory_gb,
                disk_gb: self.disk_gb,
            },
            order,
            when_nothing_fits: self.when_nothing_fits.parse().internal()?,
        })
    }
}

impl LabelStore for PgStore {
    fn create_label(&self, workspace_id: Uuid, label: &Label) -> Result<()> {
        create(self, workspace_id, label, Some(&label.order))
    }

    fn create_label_with_default_order(&self, workspace_id: Uuid, label: &Label) -> Result<()> {
        create(self, workspace_id, label, None)
    }

    fn labels(&self, workspace_id: Uuid) -> Result<Vec<Label>> {
        read_snapshot(&mut *self.connection()?, |connection| {
            load(connection, workspace_id)
        })
    }

    fn update_label(&self, workspace_id: Uuid, label: &Label) -> Result<()> {
        let fallback: &str = label.when_nothing_fits.into();
        transaction(&mut *self.connection()?, |connection| {
            lock_workspace(connection, workspace_id).store()?;
            let count = diesel::update(
                labels::table
                    .filter(labels::workspace_id.eq(workspace_id))
                    .filter(labels::deleted_at.is_null())
                    .find(label.id),
            )
            .set((
                labels::vcpus.eq(label.size.vcpus),
                labels::memory_gb.eq(label.size.memory_gb),
                labels::disk_gb.eq(label.size.disk_gb),
                labels::when_nothing_fits.eq(fallback),
            ))
            .execute(connection)
            .store()?;
            require_record(count)?;
            label_places::clear(connection, label.id).store()?;
            label_places::insert(connection, label.id, &label.order).store()
        })
    }
}

fn create(
    store: &PgStore,
    workspace_id: Uuid,
    label: &Label,
    order: Option<&[LabelPlace]>,
) -> Result<()> {
    transaction(&mut *store.connection()?, |connection| {
        if lock_workspace(connection, workspace_id)
            .optional()
            .store()?
            .is_none()
        {
            return Err(Error::InvalidReference);
        }
        insert(connection, workspace_id, label, order).store()
    })
}

/// Insert a label and its order inside the caller's workspace transaction.
pub(crate) fn insert(
    connection: &mut PgConnection,
    workspace_id: Uuid,
    label: &Label,
    order: Option<&[LabelPlace]>,
) -> QueryResult<()> {
    let fallback: &str = label.when_nothing_fits.into();
    diesel::insert_into(labels::table)
        .values((
            labels::id.eq(label.id),
            labels::workspace_id.eq(workspace_id),
            labels::name.eq(&label.name),
            labels::platform.eq(platform_text(label.platform)),
            labels::vcpus.eq(label.size.vcpus),
            labels::memory_gb.eq(label.size.memory_gb),
            labels::disk_gb.eq(label.size.disk_gb),
            labels::when_nothing_fits.eq(fallback),
        ))
        .execute(connection)?;
    if let Some(order) = order {
        return label_places::insert(connection, label.id, order);
    }
    let mut default_order = vec![LabelPlace::Machine];
    if label.platform == Platform::LinuxX64 {
        default_order.extend(
            offered_providers(connection, workspace_id)?
                .into_iter()
                .map(LabelPlace::Provider),
        );
    }
    label_places::insert(connection, label.id, &default_order)
}

/// Read active labels and their orders within the caller's snapshot.
pub(crate) fn load(connection: &mut PgConnection, workspace_id: Uuid) -> Result<Vec<Label>> {
    let mut orders = label_places::load_workspace(connection, workspace_id).store()?;
    labels::table
        .filter(labels::workspace_id.eq(workspace_id))
        .filter(labels::deleted_at.is_null())
        .order(labels::name)
        .select(LabelRow::as_select())
        .load::<LabelRow>(connection)
        .store()?
        .into_iter()
        .map(|row| {
            let order = orders.remove(&row.id).unwrap_or_default();
            row.into_record(order)
        })
        .collect()
}
