use actix_web::{error::Error as AWError};
use diesel::prelude::*;
use bcrypt::verify;
use uuid::Uuid;

use crate::schema::users;
use crate::models::{
    self,
    db::{Connection, Pool}
};
use crate::errors;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name= "users" ]
pub struct Users {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub pub_address: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlimUser {
    pub id: String,
}

impl From<Users> for SlimUser {
    fn from(user: Users) -> Self {
        SlimUser { id: user.id.to_string() }
    }
}

impl Users {
    pub fn is_pub_address_in_use(comp_pub_key: String, pool: &Pool) -> Result<bool, AWError> {
        use crate::schema::users::dsl::*;

        let conn: Connection = pool.get().unwrap();
        match users.filter(pub_address.eq(comp_pub_key)).first::<Users>(&conn) {
            Ok(_entry) => Ok(true),
            Err(_err) => Ok(false)
        }
    }

    pub fn number_of_users(pool: &Pool) -> Result<usize, errors::JuntoApiError> {
        let conn: Connection = pool.get().unwrap();
        let users_count = users::table.load::<Users>(&conn).map_err(|err| {
            println!("Error: {}", err); //log err
            errors::JuntoApiError::InternalError
        })?;
        Ok(users_count.len())
    }

    pub fn delete_all_users(pool: &Pool) -> Result<(), errors::JuntoApiError> {
        let conn: Connection = pool.get().unwrap();
        let _deletion = diesel::delete(users::table).execute(&conn).unwrap();
        Ok(())
    }

    pub fn insert_user(user: &Users, pool: &Pool) -> Result<(), AWError>{
        let conn: Connection = pool.get().unwrap();
        let _result: Users = diesel::insert_into(users::table)
            .values(user)
            .get_result(&conn)
            .unwrap();
        Ok(())
    }

    pub fn get_pub_key<'a>(user_id: &'a str, pool: &Pool) -> Result<String, errors::JuntoApiError> {
        use crate::schema::users::dsl::*;
        let user_id = Uuid::parse_str(user_id).unwrap();
        let conn: Connection = pool.get().unwrap();
        match users.select(pub_address).filter(id.eq(user_id)).first::<String>(&conn) {
            Ok(entry) => Ok(entry),
            Err(_err) => Err(errors::JuntoApiError::InternalError)
        }
    }

    pub fn can_login(auth_data: models::user::AuthData, pool: &Pool) -> Result<SlimUser, errors::JuntoApiError> {
        use crate::schema::users::dsl::*;
        let conn: Connection = pool.get().unwrap();

        let mut items = users
            .filter(email.eq(&auth_data.email))
            .load::<Users>(&conn).map_err(|_err| errors::JuntoApiError::InternalError)?;

        println!("users: {:?}", items);

        if let Some(user) = items.pop() {
            println!("items popped");
            let verify_res = verify(&auth_data.password, &user.password);
            println!("{:?}", verify_res);
            if let Ok(matching) = verify(&auth_data.password, &user.password) {
                println!("Matching");
                if matching {
                    return Ok(user.into()); // convert into slimUser
                }
            }
        }
        Err(errors::JuntoApiError::Unauthorized)
    }
}