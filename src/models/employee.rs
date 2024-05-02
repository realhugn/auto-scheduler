use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;
use diesel::{Insertable, PgConnection, prelude::*, Queryable};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::constants;
use crate::error::Error;
use crate::schema::employees;
use crate::utils::generate_token;

#[derive(Debug, Serialize, Deserialize, Queryable, PartialEq, Identifiable)]
#[diesel(table_name = employees)]
pub struct Employee {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub phone_number: Option<String>,
    pub department: Option<String>,
    pub role: String,
    pub availability: Option<Value>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = employees)]
pub struct EmployeeDTO {
    pub name: String,
    pub email: String,
    pub password: String,
    pub phone_number: Option<String>,
    pub department: Option<String>,
    pub role: String,
    pub availability: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginDTO {
    pub email: String,
    pub password : String
}

impl Employee {
    pub fn new(employee_dto: EmployeeDTO, conn: &mut PgConnection) -> Result<String, Error> {
        if Self::find_user_by_username(&employee_dto.email, conn).is_err() {
            use crate::schema::employees::dsl::*;
            let new_employee = EmployeeDTO {
                password: hash(&employee_dto.password, DEFAULT_COST).unwrap(),
                ..employee_dto
            };
            diesel::insert_into(employees).values(new_employee).execute(conn).expect(constants::DATABASE_INSERT_ERROR);
            Ok(constants::DATABASE_INSERT_SUCCESS.to_string())
        } else {
            Err(format!(
                "Employee '{}' is already registered",
                &employee_dto.email
            ).into())
        }
    }

    pub fn login(login_dto: LoginDTO, conn : &mut PgConnection) -> Result<String, Error> {
        use crate::schema::employees::dsl::*;
        let (_email, _password) = (login_dto.email.clone(), login_dto.password.clone());
        let user = employees.filter(email.eq(_email)).first::<Employee>(conn).optional()?;
        match user {
            None => return Err("User not exist".into()),
            Some(user) => {
                let db_password =user.password.clone();
                if hash(_password, DEFAULT_COST)? == db_password {
                    return Err("Password do not match".into())
                }
                let now = Utc::now();
                let access_token = generate_token(user.id.clone(), user.role.clone(), now)?;
                Ok(access_token)
            }
        }
    }


    pub fn find_user_by_username(_email: &str, conn: &mut PgConnection) -> Result<Employee, Error> {
        use crate::schema::employees::dsl::*;
        Ok(employees.filter(email.eq(_email)).first::<Employee>(conn)?)
    }

    pub fn find_by_id(_id: i32, conn: &mut PgConnection) -> Result<Employee, Error> {
        use crate::schema::employees::dsl::*;
        Ok(employees.find(_id).get_result::<Employee>(conn)?)
    }

    pub fn find_by_role(_role: &str, conn: &mut PgConnection) -> Result<Vec<Employee>, Error>{
        use crate::schema::employees::dsl::*;
        Ok(employees.filter(role.eq(_role)).get_results::<Employee>(conn)?)
    }
}