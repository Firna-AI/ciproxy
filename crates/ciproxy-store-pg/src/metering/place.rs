//! Text mapping for the place of an anonymous daily usage total.

use ciproxy_store_interface::metering::facts::JobPlace;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};

use crate::overflow_provider::DbOverflowProvider;

#[derive(AsExpression, Debug, FromSqlRow)]
#[diesel(sql_type = Text)]
pub(super) struct DbJobPlace(pub(super) JobPlace);

impl FromSql<Text, Pg> for DbJobPlace {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        if value.as_bytes() == b"machine" {
            return Ok(Self(JobPlace::Machine));
        }
        let DbOverflowProvider(provider) = DbOverflowProvider::from_sql(value)?;
        Ok(Self(JobPlace::Overflow { provider }))
    }
}

impl ToSql<Text, Pg> for DbJobPlace {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self.0 {
            JobPlace::Machine => "machine",
            JobPlace::Overflow { provider } => provider.as_str(),
        };
        <str as ToSql<Text, Pg>>::to_sql(value, out)
    }
}
