use crate::*;

#[macro_export]
macro_rules! save_bid {
	(
		$auction_bid: ident,
		$auction: ident,
		$price: ident,
		$purchaser: ident,
		$auction_id: ident,
		$AuctionBids: ident,
		$commission_agent: ident,
		$commission_data: ident,
	) => {{
		if let Some(account) = &$auction_bid.last_bid_account {
			// check the new bid price.
			let lowest_price: Balance = $auction_bid
				.last_bid_price
				.saturating_add($auction.min_raise.mul_ceil($auction_bid.last_bid_price));

			ensure!($price > lowest_price, Error::<T>::PriceTooLow);

			ensure!(&$purchaser != account, Error::<T>::DuplicatedBid);
			let _ = T::MultiCurrency::unreserve(
				$auction.currency_id,
				account,
				$auction_bid.last_bid_price,
			);
		}

		T::MultiCurrency::reserve($auction.currency_id, &$purchaser, $price)?;
		let mut auction_bid = $auction_bid;
		auction_bid.last_bid_price = $price;
		auction_bid.last_bid_account = Some($purchaser.clone());
		auction_bid.last_bid_block = frame_system::Pallet::<T>::block_number();
		auction_bid.commission_agent = $commission_agent;
		auction_bid.commission_data = $commission_data;
		$AuctionBids::<T>::insert($auction_id, auction_bid);
	}};
}

#[macro_export]
macro_rules! delete_auction {
	(
		$AuctionBids: ident,
		$Auctions: ident,
		$who: ident,
		$auction_id: ident,
		$AuctionBidNotFound: ident,
		$AuctionNotFound: ident,
	) => {
		$AuctionBids::<T>::try_mutate_exists($auction_id, |maybe_auction_bid| {
			let auction_bid =
				maybe_auction_bid.as_mut().ok_or(Error::<T>::$AuctionBidNotFound)?.clone();
			$Auctions::<T>::try_mutate_exists($who, $auction_id, |maybe_auction| {
				let auction = maybe_auction.as_mut().ok_or(Error::<T>::$AuctionNotFound)?.clone();

				if let Some(account) = &auction_bid.last_bid_account {
					let _ = T::MultiCurrency::unreserve(
						auction.currency_id,
						account,
						auction_bid.last_bid_price,
					);
				}

				let _remain: BalanceOf<T> =
					<T as Config>::Currency::unreserve(&$who, auction.deposit.saturated_into());

				for item in &auction.items {
					T::NFT::unreserve_tokens($who, item.class_id, item.token_id, item.quantity)?;
				}

				*maybe_auction_bid = None;
				*maybe_auction = None;
				Ok((auction, auction_bid))
			})
		})
	};
}

pub fn calc_current_price<T: Config>(
	max_price: Balance,
	min_price: Balance,
	created_block: BlockNumberOf<T>,
	deadline: BlockNumberOf<T>,
	current_block: BlockNumberOf<T>,
) -> Balance {
	if current_block <= created_block {
		return max_price
	} else if current_block > deadline {
		return min_price
	}

	let created_block: BlockNumber = created_block.saturated_into();
	let aligned_block: BlockNumber = current_block
		.saturated_into::<BlockNumber>()
		.saturating_sub(created_block) // >= 0
		.checked_div(DESC_INTERVAL) // >= 0
		.map(|x| x.saturating_mul(DESC_INTERVAL)) // >= 0
		.map(|x| x.saturating_add(created_block)) // >= created_block
		.unwrap_or(created_block); // >= created_block

	let deadline: FixedU128 = (deadline.saturated_into::<BlockNumber>(), 1).into();
	let created_block: FixedU128 = (created_block, 1).into();
	let current_block: FixedU128 = (aligned_block, 1).into();
	let max_price: FixedU128 = (max_price, ACCURACY).into();
	let min_price: FixedU128 = (min_price, ACCURACY).into();

	// calculate current price.
	let current_price: Balance = max_price
		.saturating_sub(min_price) // > 0
		.saturating_mul(current_block.saturating_sub(created_block)) // >= 0
		.checked_div(&deadline.saturating_sub(created_block)) // >= 0
		.map(|x| max_price.saturating_sub(x)) // >= min_price && <= max_price
		.unwrap_or(max_price) // >= min_price && <= max_price
		.saturating_mul_int(ACCURACY); // >= min_price && <= max_price

	current_price
}

pub fn get_deadline<T: Config>(
	allow_delay: bool,
	deadline: BlockNumberOf<T>,
	last_bid_block: BlockNumberOf<T>,
) -> BlockNumberFor<T> {
	if allow_delay {
		let delay = last_bid_block.saturating_add(T::ExtraConfig::auction_close_delay());
		core::cmp::max(deadline, delay)
	} else {
		deadline
	}
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
pub mod test_helper {
	use crate::*;
	use frame_support::assert_ok;
	use sp_std::vec;
	use sp_std::vec::Vec;
	use sp_std::vec;

	#[macro_export]
	macro_rules! balances {
		($amount: expr) => {
			nftmart_traits::constants_types::ACCURACY
				.saturating_mul($amount)
				.saturated_into()
		};
	}

	#[macro_export]
	macro_rules! into {
		($amount: expr) => {
			($amount as u128).saturated_into()
		};
	}

	pub fn add_whitelist<Runtime>(who: &Runtime::AccountId)
	where
		Runtime: crate::Config,
	{
		Runtime::ExtraConfig::do_add_whitelist(who);
	}

	pub fn add_class<Runtime>(who: Runtime::AccountId)
	where
		Runtime: crate::Config,
	{
		let cate_id1 = current_gid::<Runtime>();
		add_category::<Runtime>();
		let cate_id2 = current_gid::<Runtime>();
		add_category::<Runtime>();
		assert_ok!(Runtime::NFT::create_class(
			&who,
			Vec::from("1"),
			Vec::from("1"),
			Vec::from("1"),
			PerU16::from_percent(5),
			Properties(ClassProperty::Transferable | ClassProperty::Burnable),
			vec![cate_id1, cate_id2],
		));
	}

	pub fn last_event<Runtime>() -> Runtime::Event
	where
		Runtime: frame_system::Config,
	{
		frame_system::Pallet::<Runtime>::events().pop().expect("Event expected").event
	}

	pub fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
		frame_system::Pallet::<T>::assert_last_event(generic_event.into());
	}

	pub fn add_token<Runtime>(
		who: Runtime::AccountId,
		to: Runtime::AccountId,
		class_id: Runtime::ClassId,
		quantity: Runtime::TokenId,
		charge_royalty: Option<PerU16>,
	) where
		Runtime: crate::Config,
	{
		assert_ok!(Runtime::NFT::proxy_mint(
			&who,
			&to,
			class_id,
			Vec::from("1"),
			quantity,
			charge_royalty
		));
	}

	pub fn add_category<Runtime: Config>() {
		assert_ok!(Runtime::ExtraConfig::do_create_category(Vec::from("1")));
	}

	pub fn current_gid<Runtime: Config>() -> GlobalId {
		Runtime::ExtraConfig::peek_next_gid()
	}

	pub fn peek_next_class_id<Runtime: Config>() -> Runtime::ClassId {
		Runtime::NFT::peek_next_class_id()
	}
}
