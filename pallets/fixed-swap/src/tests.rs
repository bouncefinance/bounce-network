#![cfg(test)]

use crate::{mock::*, PoolDetails};
use frame_support::assert_ok;
use orml_traits::MultiReservableCurrency;
use orml_traits::MultiCurrency;

fn create_pool() {
	let creator = 0;
	let name = b"swap".to_vec();
	let token0 = 1;
	let token1 = 2;
	let total0 = 100;
	let total1 = 200;
	let duration = 50;
	assert_ok!(FixedSwap::create(
		Origin::signed(creator), name, token0, token1, total0, total1, duration
	));
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let creator = 0;
		let name = b"swap".to_vec();
		let token0 = 1;
		let token1 = 2;
		let total0 = 100;
		let total1 = 200;
		let swapped0 = 0;
		let swapped1 = 0;
		let duration = 50;
		let start_at = 0;
		let end_at = 50;
		assert_eq!(Tokens::total_issuance(token0), 100000);
		assert_eq!(Tokens::can_reserve(token0, &creator, 100000), true);
		assert_eq!(Tokens::reserved_balance(token0, &creator), 0);
		let pool_id = 0;
		let pool = PoolDetails {
			name, creator, token0, token1, total0, total1, swapped0, swapped1, duration, start_at
		};
		create_pool();
		assert_eq!(FixedSwap::pools(pool_id), pool);
		assert_eq!(FixedSwap::pool_end_at(end_at, pool_id), Some(()));
		assert_eq!(FixedSwap::next_pool_id(), 1);
		assert_eq!(Tokens::reserved_balance(token0, &creator), 100);
	});
}

#[test]
fn swap_works() {
	new_test_ext().execute_with(|| {
		create_pool();
		let creator = 0;
		let buyer = 1;
		let pool_id = 0;
		let amount1 = 20;
		let token0 = 1;
		let token1 = 2;

		assert_ok!(FixedSwap::swap(Origin::signed(buyer), pool_id, amount1));
		let pool = FixedSwap::pools(pool_id);
		assert_eq!(pool.swapped0, 10);
		assert_eq!(pool.swapped1, 20);
		assert_eq!(Tokens::reserved_balance(token0, &creator), 90);
		assert_eq!(Tokens::total_balance(token0, &buyer), 10);
		assert_eq!(Tokens::total_balance(token1, &creator), 20);
		assert_eq!(FixedSwap::swaps(pool_id, buyer), (10, 20));
	});
}

#[test]
fn auto_payout_works() {
	new_test_ext().execute_with(|| {
		create_pool();
		let creator = 0;
		let buyer = 1;
		let pool_id = 0;
		let amount1 = 20;
		let token0 = 1;
		let token1 = 2;

		assert_ok!(FixedSwap::swap(Origin::signed(buyer), pool_id, amount1));
		let pool = FixedSwap::pools(pool_id);
		assert_eq!(pool.swapped0, 10);
		assert_eq!(pool.swapped1, 20);
		assert_eq!(Tokens::reserved_balance(token0, &creator), 90);
		assert_eq!(Tokens::total_balance(token0, &buyer), 10);
		assert_eq!(Tokens::total_balance(token1, &creator), 20);
		assert_eq!(FixedSwap::swaps(pool_id, buyer), (10, 20));

		assert_eq!(Tokens::reserved_balance(token0, &creator), 90);
		FixedSwap::on_finalize(50);
		assert_eq!(Tokens::reserved_balance(token0, &creator), 0);
		assert_eq!(Tokens::total_balance(token0, &creator), 99990);
		assert_eq!(Tokens::total_balance(token0, &buyer), 10);

		assert_eq!(Tokens::total_balance(token1, &creator), 20);
		assert_eq!(Tokens::total_balance(token1, &buyer), 99980);
	});
}
