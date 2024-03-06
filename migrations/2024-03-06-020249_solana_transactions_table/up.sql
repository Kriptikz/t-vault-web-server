CREATE TABLE solana_transactions (
  id int NOT NULL AUTO_INCREMENT PRIMARY KEY,
  blockhash varchar(200) NOT NULL,
  last_valid_block_height BIGINT UNSIGNED NOT NULL,
  status SMALLINT UNSIGNED NOT NULL,
  tx varchar(2000) NOT NULL,
  created_at DATETIME(3) NOT NULL,
  updated_at DATETIME(3),
  sent_at DATETIME(3),
  confirmed_at DATETIME(3),
  finalized_at DATETIME(3),
  time_to_send INT UNSIGNED,
  time_to_confirmed INT UNSIGNED,
  time_to_finalized INT UNSIGNED,
  priority_fee INT UNSIGNED,
  tx_signature varchar(200)
);

