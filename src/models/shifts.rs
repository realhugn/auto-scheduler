use diesel::{Identifiable, Insertable, PgConnection, Queryable, QueryResult, RunQueryDsl};
use serde::{Deserialize, Serialize};
use crate::constants;
use crate::schema::shifts;
use diesel::prelude::*;


#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Clone)]
#[diesel(table_name = shifts)]
pub struct Shift {
    pub id : i32,
    pub name: String,
    pub start_time: i32 ,
    pub end_time: i32,
    pub duration: Option<i32>,
    pub minium_attendences: Option<i32>
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = shifts)]
pub struct ShiftDTO {
    pub name: String,
    pub start_time: i32,
    pub end_time: i32,
    pub duration: Option<i32>,
    pub minium_attendences: Option<i32>
}

impl Shift {
    pub fn new(shift_dto: ShiftDTO, conn: &mut PgConnection) -> Result<String, String> {
        use crate::schema::shifts::dsl::*;
        let new_shift = shift_dto;
        let rs = diesel::insert_into(shifts).values(new_shift).execute(conn);
        if rs.is_err() {
            return Err(constants::DATABASE_INSERT_ERROR.to_string())
        } else {
            Ok(constants::DATABASE_INSERT_SUCCESS.to_string())
        }
    }

    pub fn find_by_id(_id: &i32, conn: &mut PgConnection) -> QueryResult<Shift> {
        use crate::schema::shifts::dsl::*;
        shifts.find(_id).get_result::<Shift>(conn)
    }
}