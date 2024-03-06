use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Datetime, Integer, Nullable, SmallInt, Text, Unsigned};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, QueryableByName)]
#[diesel(table_name = crate::schema::solana_transactions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewSolanaTransaction {
    pub blockhash: String,
    pub last_valid_block_height: u64,
    pub status: u16,
    pub tx: String,
    pub created_at: NaiveDateTime,
    pub sent_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, QueryableByName)]
#[diesel(table_name = crate::schema::solana_transactions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct SolanaTransaction {
    pub id: i32,
    pub blockhash: String,
    pub last_valid_block_height: u64,
    pub status: u16,
    pub tx: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub sent_at: Option<NaiveDateTime>,
    pub confirmed_at: Option<NaiveDateTime>,
    pub finalized_at: Option<NaiveDateTime>,
    pub time_to_send: Option<u32>,
    pub time_to_confirmed: Option<u32>,
    pub time_to_finalized: Option<u32>,
    pub priority_fee: Option<u32>,
    pub tx_signature: Option<String>,
}

impl SolanaTransaction {
    pub async fn create(
        pool: &deadpool_diesel::mysql::Pool,
        new_tx: NewSolanaTransaction,
    ) -> Result<i32, ()> {
        let conn = pool.get().await;
        if let Ok(conn) = conn {
            let res = conn.interact(move |conn: &mut MysqlConnection| {
                diesel::sql_query("INSERT INTO solana_transactions (blockhash, last_valid_block_height, status, tx, created_at, sent_at) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind::<Text, _>(&new_tx.blockhash)
                    .bind::<Unsigned<BigInt>, _>(&new_tx.last_valid_block_height)
                    .bind::<Unsigned<SmallInt>, _>(new_tx.status)
                    .bind::<Text, _>(&new_tx.tx)
                    .bind::<Datetime, _>(new_tx.created_at)
                    .bind::<Nullable<Datetime>, _>(new_tx.sent_at)
                    .execute(conn)
                    .expect("Error inserting new transaction");

                diesel::sql_query("SELECT * FROM solana_transactions ORDER BY id DESC LIMIT 1")
                    .get_result::<SolanaTransaction>(conn)
            })
            .await;

            if let Ok(Ok(res)) = res {
                return Ok(res.id);
            }

            Err(())
        } else {
            Err(())
        }
    }

    pub async fn get_by_id(
        pool: &deadpool_diesel::mysql::Pool,
        id: i32,
    ) -> Result<SolanaTransaction, ()> {
        let conn = pool.get().await;
        if let Ok(conn) = conn {
            let res = conn
                .interact(move |conn: &mut MysqlConnection| {
                    diesel::sql_query("SELECT * FROM solana_transactions WHERE id = ?")
                        .bind::<Integer, _>(id)
                        .get_result::<SolanaTransaction>(conn)
                })
                .await;

            match res {
                Ok(Ok(transaction)) => Ok(transaction),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    pub async fn get_by_signature(
        pool: &deadpool_diesel::mysql::Pool,
        signature: String,
    ) -> Result<SolanaTransaction, ()> {
        let conn = pool.get().await;
        if let Ok(conn) = conn {
            let res = conn
                .interact(move |conn: &mut MysqlConnection| {
                    diesel::sql_query("SELECT * FROM solana_transactions WHERE tx_signature = ?")
                        .bind::<Text, _>(&signature)
                        .get_result::<SolanaTransaction>(conn)
                })
                .await;

            match res {
                Ok(Ok(transaction)) => Ok(transaction),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    pub async fn set_status_sent(
        pool: &deadpool_diesel::mysql::Pool,
        tx_id: i32,
        signature: String,
        sent_at: NaiveDateTime,
        signed_tx: String,
    ) -> Result<(), ()> {
        println!("TX ID TO UPDATE: {}", tx_id);
        let conn = pool.get().await;
        if let Ok(conn) = conn {
            let now_utc: DateTime<Utc> = Utc::now();

            let updated_at = NaiveDateTime::from_timestamp_opt(
                now_utc.timestamp(),
                now_utc.timestamp_subsec_millis() as u32 * 1_000_000,
            );

            let res = conn
                .interact(move |conn: &mut MysqlConnection| {
                    diesel::sql_query("UPDATE solana_transactions SET status = ?, sent_at = ?, tx = ?, tx_signature = ?, updated_at = ? WHERE id = ?")
                        .bind::<diesel::sql_types::Integer, _>(1) 
                        .bind::<diesel::sql_types::Nullable<Datetime>, _>(sent_at) 
                        .bind::<diesel::sql_types::Text, _>(&signed_tx)
                        .bind::<diesel::sql_types::Nullable<Text>, _>(&signature)
                        .bind::<diesel::sql_types::Nullable<Datetime>, _>(updated_at)
                        .bind::<diesel::sql_types::Integer, _>(tx_id)
                        .execute(conn)
                })
                .await;

            if let Ok(Ok(_)) = res {
                println!("Successfully updated solana_transaction in db");
                return Ok(());
            } else {
                println!("{:?}", res);
                return Err(());
            }
        } else {
            Err(())
        }
    }

    pub async fn set_status_confirmed(
        pool: &deadpool_diesel::mysql::Pool,
        id: i32,
    ) -> Result<(), ()> {
        let confirmed_at = chrono::Utc::now().naive_utc();
        SolanaTransaction::update_status(pool, id, 3, Some(confirmed_at)).await
    }

    pub async fn set_status_finalized(
        pool: &deadpool_diesel::mysql::Pool,
        id: i32,
    ) -> Result<(), ()> {
        let finalized_at = chrono::Utc::now().naive_utc();
        SolanaTransaction::update_status(pool, id, 4, Some(finalized_at)).await
    }

    async fn update_status(
        pool: &deadpool_diesel::mysql::Pool,
        id: i32,
        status: u16,
        datetime: Option<NaiveDateTime>,
    ) -> Result<(), ()> {
        let conn = pool.get().await;
        if let Ok(conn) = conn {
            let datetime_str = datetime
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                .unwrap_or_default();
            let query = match status {
                3 => format!(
                    "UPDATE solana_transactions SET status = {}, confirmed_at = '{}' WHERE id = {}",
                    status, datetime_str, id
                ),
                4 => format!(
                    "UPDATE solana_transactions SET status = {}, finalized_at = '{}' WHERE id = {}",
                    status, datetime_str, id
                ),
                _ => return Err(()), // Return an error if an unsupported status is provided
            };

            let res = conn
                .interact(move |conn: &mut MysqlConnection| diesel::sql_query(query).execute(conn))
                .await
                .map_err(|_| ());

            match res {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        } else {
            Err(())
        }
    }
}
