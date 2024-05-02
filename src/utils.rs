use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, Header};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use crate::models::schedule::{AutoScheduleDTO, DayDetail, ShiftDetail};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenClaims {
    pub sub: i32,
    pub role: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn generate_token(id: i32, role: String, _now : DateTime<Utc>) ->  Result<String, jsonwebtoken::errors::Error> {
    let iat = _now.timestamp() as usize;
    let exp = (_now + Duration::minutes(2000)).timestamp() as usize;
    let claims = TokenClaims {
        sub: id,
        role,
        iat,
        exp
    };
    let token = encode(
        &Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(include_bytes!("private_key.pem"))?
    );
    token
}

pub fn verify_jwt_token(
    token: String
) -> Result<TokenClaims, jsonwebtoken::errors::Error> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);

    let decoded = jsonwebtoken::decode::<TokenClaims>(
        token.as_str(),
        &jsonwebtoken::DecodingKey::from_rsa_pem(include_bytes!("public_key.pem"))?,
        &validation,
    );

    match decoded {
        Ok(c) => Ok(c.claims),
        Err(e) => return Err(e.into())
    }
}

pub fn create_sample_schedule(auto_schedule_dto: &AutoScheduleDTO) -> Result<Vec<DayDetail>, String> {
    let month = &auto_schedule_dto.month;
    let _year = &auto_schedule_dto.year;
    let employees: Vec<i32> = auto_schedule_dto.employees.clone();
    let days_in_month = match month {
        1|3|5|7|8|10|12 => 31,
        4|6|9|11 => 30,
        2 => 28,
        _ => return Err("Invalid Month".to_string())
    };
    let mut map : HashMap<i32, i32> = HashMap::new();
    for i in &employees {
        map.insert(*i, 0);
    }
    let mut rs: Vec<DayDetail> = Vec::new();
    let mut recent_night_employees: Vec<i32> = Vec::new();
    for i in 1..=days_in_month {
        let mut shift_detail_in_day : Vec<ShiftDetail> = Vec::new();
        let mut employees_temp = employees.clone();
        let mut rng = rand::thread_rng();
        employees_temp.shuffle(&mut rng);
        let mut number_of_employees_in_day = vec![1,1,1];
        let office_hour_employees_count = &auto_schedule_dto.nums_h;
        number_of_employees_in_day.push(*office_hour_employees_count);
        let mut shifts_indx = 0;
        for shifts in vec!["S", "C", "D", "H"] {
            let mut employees_in_this_shift: Vec<i32> = Vec::new() ;
            for _ in 0..number_of_employees_in_day[shifts_indx] {
                employees_in_this_shift.push(employees_temp.pop().unwrap());
            }
            if shifts == "S" {
                let mut index_remove : Vec<usize> = Vec::new() ;
                for i in 0..employees_in_this_shift.len() {
                    let e  = employees_in_this_shift[i];
                    if recent_night_employees.contains(&e) {
                        index_remove.push(i);
                        let mut x = employees_temp.len() - 1;
                        let mut precheck ;
                        while x > 0 {
                            precheck = employees_temp[x];
                            match recent_night_employees.contains(&precheck) {
                                true => {
                                    x -= 1;
                                }
                                false => {
                                    employees_in_this_shift.push(precheck);
                                    employees_temp.remove(x);
                                    employees_temp.push(e);
                                    break;
                                }
                            }
                        }
                    }
                }
                index_remove.sort();
                index_remove.reverse();
                for x in index_remove {
                    employees_in_this_shift.remove(x);
                }
            }
            for x in &employees_in_this_shift {
                if let Some(count) = map.get(&x) {
                    map.insert(*x, count + 1);
                } else {
                    map.insert(*x, 0);
                };
            }
            let detail_shift = ShiftDetail {
                key: shifts.to_string(),
                value: employees_in_this_shift.clone(),
            };
            shift_detail_in_day.push(detail_shift);
            if shifts == "D" {
                recent_night_employees = employees_in_this_shift.clone();
            }
            shifts_indx+=1;
        }
        let day_detail = DayDetail {
            day: i,
            value: shift_detail_in_day,
        };

        rs.push(day_detail);
    }
    println!("{:?}", map);
    println!("{:?}", rs);
    println!("{:?}", verify_valid_schedule(&rs));
    Ok(rs)
}

pub fn verify_valid_schedule(input : &Vec<DayDetail>) -> bool {
    let mut recent_night_employees: Vec<i32> = Vec::new();
    for day in input {
        let day_value = day.value.clone();
        //Because there is only 1 shift in the DayDetail have the ShiftDetail name is "S"
        let shift_filter = day_value.clone().into_iter().filter(|a| a.key == "S").collect::<Vec<ShiftDetail>>();
        let morning_shift = &shift_filter[0];
        let employees_morning: &[i32] = &morning_shift.value.clone();
        /*
            This is a O n^2 time complexity
            Although it is a bad algorithm on avarage,
            But since size of employees in each shifts is quite small (<10),
            this is the best algo to check 2 vector's elements
            Refs: https://stackoverflow.com/questions/64226562/check-if-vec-contains-all-elements-from-another-vec
        */
        if employees_morning.iter().any(|item| recent_night_employees.contains(item)) {
            return false;
        }
        let night_shift = day_value.clone().into_iter().filter(|a| a.key == "D").collect::<Vec<ShiftDetail>>();
        let night_shift = &night_shift[0];
        recent_night_employees = night_shift.value.clone();
    }

    for day in input {
        for shift in &day.value {
            if shift.value.is_empty() {
                return false;
            }
        }
    }
    return true;
}