//! Label queries and private resource row mappings.

use ciproxy_schema::labels::labels;
use ciproxy_store_interface::error::{Result, ResultExt};
use ciproxy_store_interface::labels::{Label, LabelStore};
use ciproxy_store_interface::vm_size::VmSize;
use diesel::prelude::*;
use uuid::Uuid;

use crate::connection::PgStore;
use crate::error::{QueryResultExt, require_record};

#[derive(Queryable, Selectable)]
#[diesel(table_name = labels)]
struct LabelRow {
    id: Uuid,
    name: String,
    vcpus: i32,
    memory_gb: i32,
    disk_gb: i32,
    allow_machines: bool,
    allow_boat: bool,
    when_nothing_fits: String,
}

impl LabelRow {
    fn into_record(self) -> Result<Label> {
        Ok(Label {
            id: self.id,
            name: self.name,
            size: VmSize {
                vcpus: self.vcpus,
                memory_gb: self.memory_gb,
                disk_gb: self.disk_gb,
            },
            allow_machines: self.allow_machines,
            allow_boat: self.allow_boat,
            when_nothing_fits: self.when_nothing_fits.parse().internal()?,
        })
    }
}

impl LabelStore for PgStore {
    fn create_label(&self, organization_id: Uuid, label: &Label) -> Result<()> {
        let fallback: &str = label.when_nothing_fits.into();
        diesel::insert_into(labels::table)
            .values((
                labels::id.eq(label.id),
                labels::organization_id.eq(organization_id),
                labels::name.eq(&label.name),
                labels::vcpus.eq(label.size.vcpus),
                labels::memory_gb.eq(label.size.memory_gb),
                labels::disk_gb.eq(label.size.disk_gb),
                labels::allow_machines.eq(label.allow_machines),
                labels::allow_boat.eq(label.allow_boat),
                labels::when_nothing_fits.eq(fallback),
            ))
            .execute(&mut *self.connection()?)
            .store()?;
        Ok(())
    }

    fn labels(&self, organization_id: Uuid) -> Result<Vec<Label>> {
        labels::table
            .filter(labels::organization_id.eq(organization_id))
            .order(labels::name)
            .select(LabelRow::as_select())
            .load(&mut *self.connection()?)
            .store()?
            .into_iter()
            .map(LabelRow::into_record)
            .collect()
    }

    fn update_label(&self, organization_id: Uuid, label: &Label) -> Result<()> {
        let fallback: &str = label.when_nothing_fits.into();
        let count = diesel::update(
            labels::table
                .filter(labels::organization_id.eq(organization_id))
                .find(label.id),
        )
        .set((
            labels::vcpus.eq(label.size.vcpus),
            labels::memory_gb.eq(label.size.memory_gb),
            labels::disk_gb.eq(label.size.disk_gb),
            labels::allow_machines.eq(label.allow_machines),
            labels::allow_boat.eq(label.allow_boat),
            labels::when_nothing_fits.eq(fallback),
        ))
        .execute(&mut *self.connection()?)
        .store()?;
        require_record(count)
    }
}
