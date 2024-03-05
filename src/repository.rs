use crate::schema::users;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Queryable, Selectable, QueryableByName)]
#[diesel(table_name = users)]
pub struct UserDb {
    pub id: i32,
    pub name: String,
    pub age: Option<i32>,
}

#[derive(Deserialize)]
pub struct NewUserDb {
    pub name: String,
    pub age: Option<i32>,
}

#[derive(Deserialize)]
pub struct UsersFilter {
    pub name: Option<String>,
    pub age: Option<i32>,
}

pub async fn insert(
    pool: &deadpool_diesel::mysql::Pool,
    new_user: NewUserDb,
) -> Result<UserDb, ()> {
    let conn = pool.get().await;
    if let Ok(conn) = conn {
        conn.interact(move |conn: &mut MysqlConnection| {
            diesel::sql_query("INSERT INTO users (name, age) VALUES (?, ?)")
                .bind::<diesel::sql_types::Text, _>(&new_user.name)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(&new_user.age)
                .execute(conn)
                .expect("Error inserting user");

            Ok(UserDb {
                id: 0,
                name: new_user.name.clone(),
                age: new_user.age,
            })
        })
        .await
        .map_err(|_| ())?
    } else {
        Err(())
    }
}

pub async fn get(pool: &deadpool_diesel::mysql::Pool, id: i32) -> Result<UserDb, ()> {
    let conn = pool.get().await;
    if let Ok(conn) = conn {
        let res = conn.interact(move |conn: &mut MysqlConnection| {
            diesel::sql_query("SELECT id, name, age FROM users WHERE id = ?")
                .bind::<diesel::sql_types::Integer, _>(id)
                .get_result::<UserDb>(conn)
        })
        .await
        .map_err(|_| ());

        if let Ok(Ok(user)) = res {
            return Ok(user)
        } else {
            return Err(())
        }
    } else {
        return Err(())
    }
}

pub async fn get_all(
    pool: &deadpool_diesel::mysql::Pool,
    _filter: UsersFilter,
) -> Result<Vec<UserDb>, ()> {
    let conn = pool.get().await;
    if let Ok(conn) = conn {
        let res = conn.interact(move |conn: &mut MysqlConnection| {
            let mut base_query = "SELECT id, name, age FROM users";

            let query = diesel::sql_query(base_query);

            query.load::<UserDb>(conn)
        })
        .await
        .map_err(|_| ());

        if let Ok(Ok(users)) = res {
            return Ok(users)
        } else {
            return Err(())
        }
    } else {
        Err(())
    }
}
