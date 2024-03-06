use diesel::prelude::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

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

