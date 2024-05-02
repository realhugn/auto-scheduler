use crate::utils::verify_valid_schedule;
use std::collections::HashMap;
use std::fs::OpenOptions;
use chrono::{Datelike, NaiveDate};
use diesel::{ExpressionMethods, Insertable, PgConnection, Queryable, QueryDsl, QueryResult, RunQueryDsl};
use serde::{Deserialize, Serialize};
use crate::constants;
use crate::models::employee::Employee;
use crate::models::shifts::Shift;
use crate::schema::schedules;
use crate::utils::create_sample_schedule;
use crate::error::Error;


#[derive(Serialize, Deserialize, Debug, Queryable)]
#[diesel(belongs_to(Shift, foreign_key = shift_id))]
#[diesel(belongs_to(Employee, foreign_key = employee_id))]
#[diesel(primary_key(id))]
pub struct Schedule {
    pub id: i32,
    pub employee_id: i32,
    pub data: NaiveDate,
    pub shift_id : i32,
    pub note: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Insertable)]
#[diesel(table_name = schedules)]
pub struct ScheduleDTO {
    pub employee_id: i32,
    pub data: NaiveDate,
    pub shift_id: i32,
    pub note: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShiftDetail{
    pub key: String,
    pub value: Vec<i32>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShiftDetailName{
    pub key: String,
    pub value: Vec<String>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DayDetail {
    pub day: i32,
    pub value :Vec<ShiftDetail>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DayDetailName {
    pub day: i32,
    pub value :Vec<ShiftDetailName>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AutoScheduleDTO {
    pub employees: Vec<i32>,
    pub month: i32,
    pub year: i32,
    pub nums_h: i32
}

#[allow(dead_code)]
impl Schedule {
    pub fn new(schedule_dto: ScheduleDTO, conn: &mut PgConnection) -> Result<String, Error>{
        use crate::schema::schedules::dsl::*;
        return if Employee::find_by_id(schedule_dto.employee_id, conn).is_err() && Shift::find_by_id(&schedule_dto.shift_id, conn).is_err() {
            Err(
                "Invalid employee id or shift id".into()
            )
        } else {
            let new_schedule = schedule_dto;
            let rs = diesel::insert_into(schedules).values(new_schedule).execute(conn);
            if rs.is_err() {
                return Err(
                    constants::DATABASE_INSERT_ERROR.to_string().into()
                )
            }
            Ok(constants::DATABASE_INSERT_SUCCESS.to_string())
        }
    }

    pub fn find_by_id(_id: i32, conn: &mut PgConnection) -> QueryResult<Schedule> {
        use crate::schema::schedules::dsl::*;
        schedules.find(_id).get_result::<Schedule>(conn)
    }

    /* Suppose there already have 4 shift:
        - S: Morning shift from 6.00 to 14.00 //id = 4
        - C: Afternoon shift from 2.00 to 22.00 //id = 3
        - D: Night shift from 22.00 to 6.00 next day *** //id = 2
        - H: Office hours from 8.00 to 18.00 //id = 1
     */
    pub fn from_sample_to_db(auto_schedule_dto: AutoScheduleDTO, conn: &mut PgConnection) -> Result<Vec<DayDetailName>, Error> {
        use crate::schema::schedules::dsl::*;
        use crate::schema::employees::dsl::*;
        let mut sample_schedule: Vec<DayDetail> ;
        let mut return_sample_schedule : Vec<DayDetailName> = vec![];
        let month = auto_schedule_dto.month;
        let year = auto_schedule_dto.year;

        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let schedules_in_month = schedules.filter(data.between(start_date, NaiveDate::from_ymd_opt(year, month as u32, 20).unwrap())).order_by(data).get_results::<Schedule>(conn)?;
        if schedules_in_month.len() >= 20*&auto_schedule_dto.employees.len() {
            return Err("Already generated".into())
        }
        //check if schedule is valid
        loop {
            sample_schedule = create_sample_schedule(&auto_schedule_dto).unwrap();
            if verify_valid_schedule(&sample_schedule) {
                break;
            }
        }

        // insert to datebase
        for day in &sample_schedule {
            let mut vec_detail =Vec::new();
            let _date = NaiveDate::from_ymd_opt(year, month as u32, day.day as u32).unwrap();
            for shift in &day.value {
                let key_ = shift.key.clone();

                let id_shift = match shift.key.as_str() {
                    "S" => 4,
                    "D" => 2,
                    "C" => 3,
                    "H" => 1,
                    _ => return Err(constants::DATABASE_INSERT_ERROR.to_string().into())
                };
                let mut names : Vec<String> = Vec::new();
                for uid in &shift.value {
                    let schedule = ScheduleDTO {
                        employee_id: *uid,
                        data: _date,
                        shift_id: id_shift,
                        note: None,
                    };
                    diesel::insert_into(schedules).values(schedule).execute(conn).expect("TODO: panic message");
                    let emp = employees.find(uid).get_result::<Employee>(conn).expect("Error get employee");
                    names.push(emp.name);
                }
                let _detail = ShiftDetailName {
                    key: key_,
                    value: names,
                };
                vec_detail.push(_detail);
            }
            return_sample_schedule.push(DayDetailName{
                day: day.day,
                value:vec_detail,
            });
        }

        return Ok(return_sample_schedule);
    }

    pub fn export_csv(month: i32, year: i32,  conn: &mut PgConnection) -> Result<String, Error> {
        use crate::schema::schedules::dsl::*;
        use crate::schema::employees::dsl::*;
        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let day  = match month {
            1|3|5|7|8|10|12 => 31,
            4|6|9|11 => 30,
            2 => 28,
            _ => return Err("Invalid Month".into())
        };
        let schedules_in_month = schedules.filter(data.between(start_date, NaiveDate::from_ymd_opt(year, month as u32, day).unwrap())).order_by(data).get_results::<Schedule>(conn)?;
        let nums_employees = employees.load::<Employee>(conn)?;
        let file_name = format!("schedule_{}_{}.csv", month, year);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_name.clone())?;
        let mut title = vec!["Employee Name".to_string()];
        for i in 1..=day {
            let date = format!("{}/{}/{}", i, &month, &year);
            title.push(date);
        }
        title.push("Total S".to_string());
        title.push("Total C".to_string());
        title.push("Total D".to_string());
        title.push("Total H".to_string());
        title.push("Total N".to_string());
        title.push("Total".to_string());
        let mut wtr = csv::Writer::from_writer(&file);
        wtr.write_record(&title)?;
        let mut map :HashMap<(i32, i32), String> = HashMap::new();
        for schedule in schedules_in_month {
            let shift_name :&str = match schedule.shift_id {
                1 => "H",
                2 => "D",
                3 => "C",
                4 => "S",
                _ => "N"
            };
            map.insert((schedule.data.day() as i32, schedule.employee_id) ,shift_name.to_string());
        }

        for x in nums_employees {
            let mut insert :Vec<&str> = vec![x.name.as_str()];
            for i in 1..=day {
                match map.get(&(i as i32, x.id)) {
                    Some(j) => insert.push(j),
                    None => insert.push("N")
                };
            }
            let (mut count_s, mut count_c, mut count_d, mut count_h, mut total, mut count_n) =(0, 0, 0, 0, 0, 0);
            for x in &insert {
                match x {
                    &"S" => {
                        count_s += 1;
                        total += 1;
                    },
                    &"C" => {
                        count_c += 1;
                        total += 1;
                    },
                    &"D" => {
                        count_d += 1;
                        total += 1;
                    },
                    &"H" => {
                        count_h += 1;
                        total += 1;
                    },
                    _ => {
                        count_n += 1;
                    }
                }
            }
            let (count_s_str, count_c_str, count_d_str, count_h_str,  total_str , count_n_str) = (
                    count_s.to_string(),
                    count_c.to_string(),
                    count_d.to_string(),
                    count_h.to_string(),
                    total.to_string(),
                    count_n.to_string()
                );
            insert.push(&*count_s_str);
            insert.push(&*count_c_str);
            insert.push(&*count_d_str);
            insert.push(&*count_h_str);
            insert.push(&*count_n_str);
            insert.push(&*total_str);
            wtr.write_record(insert)?;
        }
        wtr.flush()?;
        let mut rdr = csv::Reader::from_reader(&file);
        for result in rdr.records() {
            let record = result?;
            println!("{:?}", record);
        }
        Ok(file_name)
    }

    pub fn get_by_month_year(month: i32, year: i32,  conn: &mut PgConnection) -> Result<Vec<DayDetailName>, Error> {
        use crate::schema::schedules::dsl::*;
        use crate::schema::employees::dsl::*;
        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let day  = match month {
            1|3|5|7|8|10|12 => 31,
            2 => 29,
            4|6|9|11 => 30,
            _ => return Err("Invalid Month".into())
        };
        let schedules_in_month = schedules.filter(data.between(start_date, NaiveDate::from_ymd_opt(year, month as u32, day).unwrap())).order_by(data).get_results::<Schedule>(conn)?;
        let nums_employees = employees.load::<Employee>(conn)?;
        let mut map_id_name : HashMap<i32, String> = HashMap::new();

        for emp in nums_employees {
            map_id_name.insert(emp.id, emp.name);
        }
        let mut map :HashMap<(i32, String), Vec<String>> = HashMap::new();
        let mut rs : Vec<DayDetailName> = Vec::new();
        for schedule in schedules_in_month {

            let shift_name :&str = match schedule.shift_id {
                1 => "H",
                2 => "D",
                3 => "C",
                4 => "S",
                _ => "N"
            };
            let vl = map_id_name.get(&schedule.employee_id).unwrap().clone();
            match map.entry((schedule.data.day() as i32, shift_name.to_string())) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().push(vl);
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(vec![vl]);
                }
            }
        }
        for i in 1..=day {
            let mut vec_shift : Vec<ShiftDetailName> = Vec::new();
            for shift in vec!["S", "C", "D", "H"] {
                if month == 2 && year == 2024 && i == 29 {
                    vec_shift.push(
                        ShiftDetailName {
                        key: shift.to_string(),
                        value: vec![],
                    });
                    continue;
                }
                let insert : Vec<String> = map.get(&(i as i32, shift.to_string())).unwrap_or(&vec![]).clone();
                let shift_value = ShiftDetailName {
                    key: shift.to_string(),
                    value: insert,
                };
                vec_shift.push(shift_value)
            }
            let day_detail = DayDetailName {
                day: i as i32,
                value: vec_shift,
            };
            rs.push(day_detail);
        }
        Ok(rs)
    }
    //
    fn analysis(month: i32, year: i32,  conn: &mut PgConnection) -> Result<String, Error> {

        Ok("Oke".to_string())
    }
}



#[cfg(test)]
mod tests {
    use crate::models::schedule::{AutoScheduleDTO, create_sample_schedule, Schedule};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_automate_create_schedule() {
        let dto = AutoScheduleDTO {
            employees: vec![1,2,3,5,7,8,9,11],
            month: 1,
            year: 2024,
            nums_h: 2
        };
        let rs = create_sample_schedule(&dto);
        assert!(rs.is_ok())
    }

    // #[test]
    // fn test_export() {
    //     let month = 1;
    //     let year = 2024;
    //     let rs = Schedule::export_csv(month, year);
    //     assert!(rs.is_ok())
    // }

}