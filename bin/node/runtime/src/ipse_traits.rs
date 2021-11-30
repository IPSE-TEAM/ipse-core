pub trait PocHandler<AccountId> {
	fn remove_history(miner: AccountId);
}

pub trait GetPrice<Balance> {
	fn get_price() -> Balance;
}
