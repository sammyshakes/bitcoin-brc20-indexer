pub const COLLECTION_TICKERS: &str = "brc20_tickers";
pub const COLLECTION_DEPLOYS: &str = "brc20_deploys";
pub const COLLECTION_MINTS: &str = "brc20_mints";
pub const COLLECTION_TRANSFERS: &str = "brc20_transfers";
pub const COLLECTION_INVALIDS: &str = "brc20_invalids";
pub const COLLECTION_USER_BALANCES: &str = "brc20_user_balances";
pub const COLLECTION_USER_BALANCE_ENTRY: &str = "brc20_user_balance_entry";
pub const COLLECTION_BLOCKS_COMPLETED: &str = "blocks_completed";
pub const COLLECTION_BRC20_ACTIVE_TRANSFERS: &str = "brc20_active_transfers";
pub const COLLECTION_TOTAL_MINTED_AT_BLOCK_HEIGHT: &str = "total_minted_at_block_height";
pub const MONGO_RETRIES: u32 = 10000000;

// testnet 2423500 mainnet 767430
pub const BRC20_STARTING_BLOCK_HEIGHT: i64 = 2423500;
pub const KEY_BLOCK_HEIGHT: &str = "block_height";
pub const OVERALL_BALANCE: &str = "overall_balance";
pub const TRANSFERABLE_BALANCE: &str = "transferable_balance";
pub const AVAILABLE_BALANCE: &str = "available_balance";
