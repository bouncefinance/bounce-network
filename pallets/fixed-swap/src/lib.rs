#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_runtime::{
	RuntimeDebug,
	traits::{
		MaybeSerializeDeserialize, Member, AtLeast32BitUnsigned, Saturating, Zero,
	}
};
use sp_std::{fmt::Debug, prelude::Vec};
use frame_support::{
	ensure, decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult, weights::Weight, Parameter,
};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiReservableCurrency};

mod default_weight;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct PoolDetails<
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq + Default,
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq + Default,
	BlockNumber: Encode + Decode + Clone + Debug + Eq + PartialEq + Default,
	TokenId: Encode + Decode + Clone + Debug + Eq + PartialEq + Default,
> {
	name: Vec<u8>,
	creator: AccountId,
	token0: TokenId,
	token1: TokenId,
	total0: Balance,
	total1: Balance,
	swapped0: Balance,
	swapped1: Balance,
	duration: BlockNumber,
	start_at: BlockNumber,
}

pub trait WeightInfo {
	fn create() -> Weight;
	fn swap() -> Weight;
	fn on_finalize(count: u32) -> Weight;
}

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	/// The units in which we record balances.
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

	/// The arithmetic type of pool identifier.
	type PoolId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	/// The type of token identifier.
	type TokenId: Member + Parameter + Default + Copy + MaybeSerializeDeserialize;

	/// The currency mechanism.
	type Currency: MultiCurrency<Self::AccountId, CurrencyId = Self::TokenId, Balance = Self::Balance>
		+ MultiReservableCurrency<Self::AccountId>;

	/// Weight information for extrinsics in this module.
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Config> as FixedSwap {
		// Next id of a pool
		NextPoolId get(fn next_pool_id): T::PoolId;

		/// Details of a pool.
		Pool: map hasher(blake2_128_concat) T::PoolId
			=> PoolDetails<T::AccountId, T::Balance, T::BlockNumber, T::TokenId>;

		/// Swap records by a pool and an account.
		Swap: double_map hasher(blake2_128_concat) T::PoolId, hasher(blake2_128_concat) T::AccountId
			=> (T::Balance, T::Balance);

		/// The end block number of a pool
		PoolEndAt get(fn pool_end_at):
			double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) T::PoolId
			=> Option<()>;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
		<T as Config>::PoolId,
	{
		PoolCreated(PoolId, AccountId),
		PoolSwapped(PoolId, AccountId),
		PoolClosed(PoolId),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		InvalidDuration,
		PoolExpired,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = T::WeightInfo::create()]
		pub fn create(
			origin,
			name: Vec<u8>,
			token0: T::TokenId,
			token1: T::TokenId,
			total0: T::Balance,
			total1: T::Balance,
			duration: T::BlockNumber,
		) {
			ensure!(duration > Zero::zero(), Error::<T>::InvalidDuration);

			let creator = ensure_signed(origin)?;
			let pool_id: T::PoolId = NextPoolId::<T>::get();
			let start_at = <frame_system::Module<T>>::block_number();
			let end_at = start_at.saturating_add(duration);

			T::Currency::reserve(token0, &creator, total0)?;

			Pool::<T>::insert(pool_id, PoolDetails {
				name,
				creator: creator.clone(),
				token0,
				token1,
				total0,
				total1,
				swapped0: Zero::zero(),
				swapped1: Zero::zero(),
				duration,
				start_at,
			});
			PoolEndAt::<T>::insert(end_at, pool_id, ());
			NextPoolId::<T>::put(pool_id.saturating_add(1u32.into()));

			Self::deposit_event(RawEvent::PoolCreated(pool_id, creator));
		}

		#[weight = T::WeightInfo::swap()]
		pub fn swap(
			origin,
			pool_id: T::PoolId,
			amount1: T::Balance,
		) {
			let buyer = ensure_signed(origin)?;

			Pool::<T>::try_mutate(pool_id, |pool| -> DispatchResult {
				let now = <frame_system::Module<T>>::block_number();
				ensure!(pool.start_at.saturating_add(pool.duration) < now, Error::<T>::PoolExpired);

				let amount0 = amount1.saturating_mul(pool.total0) / pool.total1;
				pool.swapped0 = pool.swapped0.saturating_add(amount0);
				pool.swapped1 = pool.swapped1.saturating_add(amount1);

				T::Currency::unreserve(pool.token0, &pool.creator, amount0);
				T::Currency::transfer(pool.token0, &pool.creator, &buyer, amount0)?;
				T::Currency::transfer(pool.token1, &buyer, &pool.creator, amount0)?;

				Swap::<T>::try_mutate(pool_id, &buyer, |swap| -> DispatchResult {
					swap.0 = swap.0.saturating_mul(amount0);
					swap.1 = swap.1.saturating_mul(amount1);
					Ok(())
				})?;

				Self::deposit_event(RawEvent::PoolSwapped(pool_id, buyer));
				Ok(())
			})?;
		}

		fn on_initialize(now: T::BlockNumber) -> Weight {
			T::WeightInfo::on_finalize(PoolEndAt::<T>::iter_prefix(&now).count() as u32)
		}

		fn on_finalize(now: T::BlockNumber) {
			Self::on_finalize(now);
		}
	}
}

impl<T: Config> Module<T> {
	fn on_finalize(now: T::BlockNumber) {
		for (pool_id, _) in PoolEndAt::<T>::drain_prefix(&now) {
			let pool = Pool::<T>::get(pool_id);
			let un_swapped0 = pool.total0.saturating_sub(pool.swapped0);
			if un_swapped0 > Zero::zero() {
				T::Currency::unreserve(pool.token0, &pool.creator, un_swapped0);
			}
			Self::deposit_event(RawEvent::PoolClosed(pool_id));
		}
	}
}
