use diesel::{Identifiable, Insertable, PgConnection, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use crate::constants;
use crate::error::Error;
use crate::schema::shift_changes;
use diesel::prelude::*;


#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
pub struct ShiftChange {
    pub id: i32,
    pub scheduler_id : i32,
    pub reason: Option<String>,
    pub status: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = shift_changes)]
pub struct ShiftChangeDTO {
    pub scheduler_id : i32,
    pub reason : Option<String>
}

impl ShiftChange {
    pub fn new(shift_change_dto: ShiftChangeDTO, conn: &mut PgConnection) -> Result<String, Error> {
        use crate::schema::shift_changes::dsl::*;

        diesel::insert_into(shift_changes).values(&shift_change_dto).execute(conn).expect(constants::DATABASE_INSERT_ERROR);
        Ok(constants::DATABASE_INSERT_SUCCESS.to_string())
    }

    pub fn verify_change(shift_change_id: i32, conn: &mut PgConnection) -> Result<String ,Error> {
        use crate::schema::shift_changes::dsl::*;

        let _shift_change = diesel::update(shift_changes.find(shift_change_id)).set(status.eq("Ok")).get_result::<ShiftChange>(conn).expect(constants::DATABASE_UPDATE_ERROR);

        // delete schedule when verify change ( service module will be created later to seperate logic)
        Ok(constants::DATABASE_INSERT_SUCCESS.to_string())
    }
}