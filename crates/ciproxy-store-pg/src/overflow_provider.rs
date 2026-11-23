//! PostgreSQL text mapping for overflow provider identifiers.

use ciproxy_store_interface::overflow::provider::OverflowProvider;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};

#[derive(AsExpression, Clone, Copy, Debug, FromSqlRow)]
#[diesel(sql_type = Text)]
pub(crate) struct DbOverflowProvider(pub(crate) OverflowProvider);

impl FromSql<Text, Pg> for DbOverflowProvider {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        match value.as_bytes() {
            b"boat" => Ok(Self(OverflowProvider::Boat)),
            _ => Err("unknown overflow provider".into()),
        }
    }
}

impl ToSql<Text, Pg> for DbOverflowProvider {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <str as ToSql<Text, Pg>>::to_sql(self.0.as_str(), out)
    }
}
