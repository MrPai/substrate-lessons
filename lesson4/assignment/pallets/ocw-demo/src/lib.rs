//! A demonstration of an offchain worker that sends onchain callbacks

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use core::{convert::TryInto, fmt};
use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
};
use parity_scale_codec::{Decode, Encode};

use frame_system::{
	self as system, ensure_none, ensure_signed,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
		SignedPayload, SigningTypes, Signer, SubmitTransaction,
	},
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	RuntimeDebug,
	offchain as rt_offchain,
	offchain::{
		storage::StorageValueRef,
		storage_lock::{StorageLock, BlockAndTime},
	},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity,
		ValidTransaction,
	},
};
use sp_std::{
	prelude::*, str,
	collections::vec_deque::VecDeque,
};

use serde::{Deserialize, Deserializer};

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When an offchain worker is signing transactions it's going to request keys from type
/// `KeyTypeId` via the keystore to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");
pub const NUM_VEC_LEN: usize = 10;
/// The type to sign and send transactions.
pub const UNSIGNED_TXS_PRIORITY: u64 = 100;

// We are fetching information from the github public API about organization`substrate-developer-hub`.
pub const HTTP_REMOTE_REQUEST: &str = "https://api.coincap.io/v2/assets/polkadot";
pub const HTTP_HEADER_USER_AGENT: &str = "MrPai";

pub const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds
pub const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD + 1000; // in milli-seconds
//如果超过2/3的validator已经发送了报价，那么再经过3个区块高度，开始计算当前轮次的平均价格
pub const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrapper.
/// We can utilize the supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// them with the pallet-specific identifier.
pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	// implemented for ocw-runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

// #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
// pub struct Payload<Public> {
// 	number: u32,
// 	public: Public
// }

// impl <T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
// 	fn public(&self) -> T::Public {
// 		self.public.clone()
// 	}
// }

// #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
// pub struct PricePayload<Public> {
// 	price: String,
// 	public: Public
// }

// impl <T: SigningTypes> SignedPayload<T> for PricePayload<T::Public> {
// 	fn public(&self) -> T::Public {
// 		self.public.clone()
// 	}
// }

// ref: https://serde.rs/container-attrs.html#crate
// #[derive(Deserialize, Encode, Decode, Default)]
// struct GithubInfo {
// 	// Specify our own deserializing function to convert JSON string to vector of bytes
// 	#[serde(deserialize_with = "de_string_to_bytes")]
// 	login: Vec<u8>,
// 	#[serde(deserialize_with = "de_string_to_bytes")]
// 	blog: Vec<u8>,
// 	public_repos: u32,
// }

pub type AuthIndex = u32;
pub type Round = u32;
pub type Price = u64;
#[derive(Deserialize, Encode, Decode, Default)]
struct DOTUSDJson {
	data: DataDetail,
	timestamp: u64,
}

#[derive(Deserialize, Encode, Decode, Default)]
struct DataDetail {
	#[serde(deserialize_with = "de_string_to_bytes")]
	id: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	symbol: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	priceUsd: Vec<u8>,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

// impl fmt::Debug for GithubInfo {
// 	// `fmt` converts the vector of bytes inside the struct back to string for
// 	//   more friendly display.
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		write!(
// 			f,
// 			"{{ login: {}, blog: {}, public_repos: {} }}",
// 			str::from_utf8(&self.login).map_err(|_| fmt::Error)?,
// 			str::from_utf8(&self.blog).map_err(|_| fmt::Error)?,
// 			&self.public_repos
// 		)
// 	}
// }

impl fmt::Debug for DOTUSDJson {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{{ data: {}, timestamp: {} }}",
			&self.data,
			&self.timestamp
		)
	}
}

impl fmt::Debug for DataDetail {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{{ id: {}, symbol: {}, priceUsd: {} }}",
			str::from_utf8(&self.id).map_err(|_| fmt::Error)?,
			str::from_utf8(&self.symbol).map_err(|_| fmt::Error)?,
			str::from_utf8(&self.priceUsd).map_err(|_| fmt::Error)?
		)
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct CommitPrice<BlockNumber>
	where BlockNumber: PartialEq + Eq + Decode + Encode,
{
	/// 当前价格提交的轮次
	pub round: Round,
	/// 提交的价格
	pub price: Price,
	/// 提交时的区块高度
	pub block_number: BlockNumber,
	/// Index of the current session.
	pub session_index: SessionIndex,
	/// The length of session validator set
	pub validators_len: u32,
}

/// This is the pallet's configuration trait
pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> {
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	///用来表示小数位精度，每一个f64将乘以这个常量后转化为u64，剩下的小数位将舍弃
	type PricePrecision: Get<u8>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Example {
		/// 当前价格的轮次，在计算平均价格时，各个validator提交的价格round必须一致，每次递增1
		/// 每次提交价格的验证者节点数量必须大于2/3，才可以进行下一个round的价格抓取
		Round get(fn round_count): Round;

		/// 如果当前轮次中，超过2/3的validator已经提交报价，那么记录这“第2/3个”validator提交价格时的区块高度
		/// 当区块高度再经过“LOCK_BLOCK_EXPIRATION”个后，开始计算平均价格;
		/// 这也会是offchain_worker回调里首先进行的逻辑验证
		BlockHeight get(fn block_height): (Round, T::BlockNumber);

		/// A vector of recently submitted numbers. Bounded by NUM_VEC_LEN
		// Numbers get(fn numbers): VecDeque<u32>;
		///储存各个validator提交上的报价，取平均价作为实际价格
		ReceivedPrices get(fn received_prices):
			double_map hasher(twox_64_concat) SessionIndex, hasher(twox_64_concat) AuthIndex
			=> Option<(Round,Price)>;
		///储存平均价格的双向链表，只保留NUM_VEC_LEN位
		AveragePrice get(fn average_price): VecDeque<(Round,Price)>;
	}
}

decl_event!(
	/// Events generated by the module.
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		/// Event generated when a new number is accepted to contribute to the average.
		// NewNumber(Option<AccountId>, u32),

		///报价节点的Id，round as u64，price as u64
		ReceivedPrice(AccountId, Round, Price),
		///当满足指定区块高度开始计算平均价格时，触发
		AveragePrice(T::BlockNumber, Round, Price),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		// Error returned when not sure which ocw function to executed
		UnknownOffchainMux,

		// Error returned when making signed transactions in off-chain worker
		NoLocalAcctForSigning,
		OffchainSignedTxError,

		// Error returned when making unsigned transactions in off-chain worker
		OffchainUnsignedTxError,

		// Error returned when making unsigned transactions with signed payloads in off-chain worker
		OffchainUnsignedTxSignedPayloadError,

		// Error returned when fetching github info
		HttpFetchingError,

		AcquireStorageLockError,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		// #[weight = 10000]
		// pub fn submit_number_signed(origin, number: u32) -> DispatchResult {
		// 	let who = ensure_signed(origin)?;
		// 	debug::info!("submit_number_signed: ({}, {:?})", number, who);
		// 	Self::append_or_replace_number(number);

		// 	Self::deposit_event(RawEvent::NewNumber(Some(who), number));
		// 	Ok(())
		// }

		// #[weight = 10000]
		// pub fn submit_number_unsigned(origin, number: u32) -> DispatchResult {
		// 	let _ = ensure_none(origin)?;
		// 	debug::info!("submit_number_unsigned: {}", number);
		// 	Self::append_or_replace_number(number);

		// 	Self::deposit_event(RawEvent::NewNumber(None, number));
		// 	Ok(())
		// }

		// #[weight = 10000]
		// pub fn submit_number_unsigned_with_signed_payload(origin, payload: Payload<T::Public>,
		// 	_signature: T::Signature) -> DispatchResult
		// {
		// 	let _ = ensure_none(origin)?;
		// 	// we don't need to verify the signature here because it has been verified in
		// 	//   `validate_unsigned` function when sending out the unsigned tx.
		// 	let Payload { number, public } = payload;
		// 	debug::info!("submit_number_unsigned_with_signed_payload: ({}, {:?})", number, public);
		// 	Self::append_or_replace_number(number);

		// 	Self::deposit_event(RawEvent::NewNumber(None, number));
		// 	Ok(())
		// }


		#[weight = 10000]
		pub fn submit_price_unsigned_with_signed_payload(origin, payload: PricePayload<T::Public>,
			_signature: T::Signature) -> DispatchResult
		{
			/// Price Oracle
			///每个validator都提交自己的价格到链上，当超过2/3的validator成功提交后既可以计算价格平均价
			///只有validator才能提交，因此使用“unsigned_with_signed_payload”.
			let who = ensure_none(origin)?;
			// we don't need to verify the signature here because it has been verified in
			//   `validate_unsigned` function when sending out the unsigned tx.
			let PricePayload { price, public } = payload;
			debug::info!("submit_price_unsigned_with_signed_payload: ({}, {:?})", price, public);
			Self::append_or_replace_price(price);

			Self::deposit_event(RawEvent::NewPrice(Some(who), price));
			Ok(())
		}

		#[weight = 10000]
		pub fn calculate_average_price_unsigned_with_signed_payload(origin, payload: PricePayload<T::Public>,
			_signature: T::Signature) -> DispatchResult
		{
			//todo 如果满足条件，则计算平均价格

		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("Entering off-chain worker");
			let round = Self::round_count();
			//TODO 首先验证是否满足计算平均价格的条件


			// Here we are showcasing various techniques used when running off-chain workers (ocw)
			// 1. Sending signed transaction from ocw
			// 2. Sending unsigned transaction from ocw
			// 3. Sending unsigned transactions with signed payloads from ocw
			// 4. Fetching JSON via http requests in ocw
			// const TX_TYPES: u32 = 4;
			// let modu = block_number.try_into().map_or(TX_TYPES, |bn: u32| bn % TX_TYPES);
			// let result = match modu {
			// 	0 => Self::offchain_signed_tx(block_number),
			// 	1 => Self::offchain_unsigned_tx(block_number),
			// 	2 => Self::offchain_unsigned_tx_signed_payload(block_number),
			// 	3 => Self::fetch_github_info(),
			// 	_ => Err(Error::<T>::UnknownOffchainMux),
			// };

			// if let Err(e) = result {
			// 	debug::error!("offchain_worker error: {:?}", e);
			// }
			
			match Self::fetch_DOTUSD_price(round) {
				Ok(result: DOTUSDJson) => {
					/// 查询链上数据，
					/// 如果当前轮次当前节点已经提交过，则不重复提交
					

					/// 如果当前轮次没有提交
					Self::offchain_price_unsigned_with_signed_payload(block_number, result);

				},
				Err(e) => {
					debug::error!("offchain_worker error: {:?}", e);
				}
			}
		}
	}
}

impl<T: Trait> Module<T> {
	/// Append a new number to the tail of the list, removing an element from the head if reaching
	///   the bounded length.
	// fn append_or_replace_number(number: u32) {
	// 	Numbers::mutate(|numbers| {
	// 		if numbers.len() == NUM_VEC_LEN {
	// 			let _ = numbers.pop_front();
	// 		}
	// 		numbers.push_back(number);
	// 		debug::info!("Number vector: {:?}", numbers);
	// 	});
	// }

	// fn append_or_replace_price(price: String) {
	// 	Prices::mutate(|prices| {
	// 		if prices.len() == NUM_VEC_LEN {
	// 			let _ = prices.pop_front();
	// 		}
	// 		prices.push_back(price);
	// 		debug::info!("Number vector: {:?}", prices);
	// 	});
	// }

	/// Check if we have fetched github info before. If yes, we can use the cached version
	///   stored in off-chain worker storage `storage`. If not, we fetch the remote info and
	///   write the info into the storage for future retrieval.
	// fn fetch_github_info() -> Result<(), Error<T>> {
	// 	// Create a reference to Local Storage value.
	// 	// Since the local storage is common for all offchain workers, it's a good practice
	// 	// to prepend our entry with the pallet name.
	// 	let s_info = StorageValueRef::persistent(b"offchain-demo::gh-info");

	// 	// Local storage is persisted and shared between runs of the offchain workers,
	// 	// offchain workers may run concurrently. We can use the `mutate` function to
	// 	// write a storage entry in an atomic fashion.
	// 	//
	// 	// With a similar API as `StorageValue` with the variables `get`, `set`, `mutate`.
	// 	// We will likely want to use `mutate` to access
	// 	// the storage comprehensively.
	// 	//
	// 	// Ref: https://substrate.dev/rustdocs/v2.0.0/sp_runtime/offchain/storage/struct.StorageValueRef.html
	// 	if let Some(Some(gh_info)) = s_info.get::<GithubInfo>() {
	// 		// gh-info has already been fetched. Return early.
	// 		debug::info!("cached gh-info: {:?}", gh_info);
	// 		return Ok(());
	// 	}

		// Since off-chain storage can be accessed by off-chain workers from multiple runs, it is important to lock
		//   it before doing heavy computations or write operations.
		// ref: https://substrate.dev/rustdocs/v2.0.0-rc3/sp_runtime/offchain/storage_lock/index.html
		//
		// There are four ways of defining a lock:
		//   1) `new` - lock with default time and block exipration
		//   2) `with_deadline` - lock with default block but custom time expiration
		//   3) `with_block_deadline` - lock with default time but custom block expiration
		//   4) `with_block_and_time_deadline` - lock with custom time and block expiration
		// Here we choose the most custom one for demonstration purpose.
		// let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
		// 	b"offchain-demo::lock", LOCK_BLOCK_EXPIRATION,
		// 	rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION)
		// );

		// We try to acquire the lock here. If failed, we know the `fetch_n_parse` part inside is being
		//   executed by previous run of ocw, so the function just returns.
		// ref: https://substrate.dev/rustdocs/v2.0.0/sp_runtime/offchain/storage_lock/struct.StorageLock.html#method.try_lock
	// 	if let Ok(_guard) = lock.try_lock() {
	// 		match Self::fetch_n_parse() {
	// 			Ok(gh_info) => { s_info.set(&gh_info); }
	// 			Err(err) => { return Err(err); }
	// 		}
	// 	}
	// 	Ok(())
	// }

	fn fetch_DOTUSD_price(round: Round) -> Result<DOTUSDJson, Error<T>> {
		
		let s_info = StorageValueRef::persistent(round.to_be_bytes());
		if let Some(Some(gh_info)) = s_info.get::<DOTUSDJson>() {
			debug::info!("cached gh-info: {:?}", gh_info);
			return Ok(gh_info);
		}

		let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
			b"offchain-demo::lock", LOCK_BLOCK_EXPIRATION,
			rt_offchain::Duration::from_millis(LOCK_TIMEOUT_EXPIRATION)
		);

		if let Ok(_guard) = lock.try_lock() {
			match Self::fetch_n_parse() {
				Ok(gh_info) => { 
					s_info.set(&gh_info);
					return Ok(gh_info);
				}
				Err(err) => { 
					return Err(err); 
				}
			}
		}
		Err(<Error<T>>::AcquireStorageLockError.into())
	}

	/// Fetch from remote and deserialize the JSON to a struct
	fn fetch_n_parse() -> Result<GithubInfo, Error<T>> {
		let resp_bytes = Self::fetch_from_remote().map_err(|e| {
			debug::error!("fetch_from_remote error: {:?}", e);
			<Error<T>>::HttpFetchingError
		})?;

		let resp_str = str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
		// Print out our fetched JSON string
		debug::info!("{}", resp_str);

		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
		// let gh_info: GithubInfo =
		// 	serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError)?;
		// Ok(gh_info)

		let gh_info: DOTUSDJson =
			serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError)?;
		Ok(gh_info)
	}

	/// This function uses the `offchain::http` API to query the remote github information,
	///   and returns the JSON response as vector of bytes.
	fn fetch_from_remote() -> Result<Vec<u8>, Error<T>> {
		debug::info!("sending request to: {}", HTTP_REMOTE_REQUEST);

		// Initiate an external HTTP GET request. This is using high-level wrappers from `sp_runtime`.
		let request = rt_offchain::http::Request::get(HTTP_REMOTE_REQUEST);

		// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
		let timeout = sp_io::offchain::timestamp()
			.add(rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD));

		// For github API request, we also need to specify `user-agent` in http request header.
		//   See: https://developer.github.com/v3/#user-agent-required
		let pending = request
			.add_header("User-Agent", HTTP_HEADER_USER_AGENT)
			.deadline(timeout) // Setting the timeout time
			.send() // Sending the request out by the host
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		// By default, the http request is async from the runtime perspective. So we are asking the
		//   runtime to wait here.
		// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
		//   ref: https://substrate.dev/rustdocs/v2.0.0/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
		let response = pending
			.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError)?
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError);
		}

		// Next we fully read the response body and collect it to a vector of bytes.
		Ok(response.body().collect::<Vec<u8>>())
	}

	// fn offchain_signed_tx(block_number: T::BlockNumber) -> Result<(), Error<T>> {
	// 	// We retrieve a signer and check if it is valid.
	// 	//   Since this pallet only has one key in the keystore. We use `any_account()1 to
	// 	//   retrieve it. If there are multiple keys and we want to pinpoint it, `with_filter()` can be chained,
	// 	//   ref: https://substrate.dev/rustdocs/v2.0.0/frame_system/offchain/struct.Signer.html
	// 	let signer = Signer::<T, T::AuthorityId>::any_account();

	// 	// Translating the current block number to number and submit it on-chain
	// 	let number: u32 = block_number.try_into().unwrap_or(0);

	// 	// `result` is in the type of `Option<(Account<T>, Result<(), ()>)>`. It is:
	// 	//   - `None`: no account is available for sending transaction
	// 	//   - `Some((account, Ok(())))`: transaction is successfully sent
	// 	//   - `Some((account, Err(())))`: error occured when sending the transaction
	// 	let result = signer.send_signed_transaction(|_acct|
	// 		// This is the on-chain function
	// 		Call::submit_number_signed(number)
	// 	);

	// 	// Display error if the signed tx fails.
	// 	if let Some((acc, res)) = result {
	// 		if res.is_err() {
	// 			debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
	// 			return Err(<Error<T>>::OffchainSignedTxError);
	// 		}
	// 		// Transaction is sent successfully
	// 		return Ok(());
	// 	}

	// 	// The case of `None`: no account is available for sending
	// 	debug::error!("No local account available");
	// 	Err(<Error<T>>::NoLocalAcctForSigning)
	// }

	// fn offchain_unsigned_tx(block_number: T::BlockNumber) -> Result<(), Error<T>> {
	// 	let number: u32 = block_number.try_into().unwrap_or(0);
	// 	let call = Call::submit_number_unsigned(number);

	// 	// `submit_unsigned_transaction` returns a type of `Result<(), ()>`
	// 	//   ref: https://substrate.dev/rustdocs/v2.0.0/frame_system/offchain/struct.SubmitTransaction.html#method.submit_unsigned_transaction
	// 	SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
	// 		.map_err(|_| {
	// 			debug::error!("Failed in offchain_unsigned_tx");
	// 			<Error<T>>::OffchainUnsignedTxError
	// 		})
	// }

	fn offchain_unsigned_tx_signed_payload(block_number: T::BlockNumber) -> Result<(), Error<T>> {
		// Retrieve the signer to sign the payload
		let signer = Signer::<T, T::AuthorityId>::any_account();

		let number: u32 = block_number.try_into().unwrap_or(0);

		// `send_unsigned_transaction` is returning a type of `Option<(Account<T>, Result<(), ()>)>`.
		//   Similar to `send_signed_transaction`, they account for:
		//   - `None`: no account is available for sending transaction
		//   - `Some((account, Ok(())))`: transaction is successfully sent
		//   - `Some((account, Err(())))`: error occured when sending the transaction
		if let Some((_, res)) = signer.send_unsigned_transaction(
			|acct| Payload { number, public: acct.public.clone() },
			Call::submit_number_unsigned_with_signed_payload
		) {
			return res.map_err(|_| {
				debug::error!("Failed in offchain_unsigned_tx_signed_payload");
				<Error<T>>::OffchainUnsignedTxSignedPayloadError
			});
		}

		// The case of `None`: no account is available for sending
		debug::error!("No local account available");
		Err(<Error<T>>::NoLocalAcctForSigning)
	}

	fn offchain_price_unsigned_with_signed_payload(block_number: T::BlockNumber, json: DOTUSDJson){
		//TODO START FROM HERE
		//按照目前的设计思路，需要引入pallet_session，实现较为复杂
		//本次版本的代码保留，当把目前作业快点做完就好了
		
		let session_index = <pallet_session::Module<T>>::current_index();
		let validators_len = <pallet_session::Module<T>>::validators().len() as u32;

		let exists = <ReceivedPrices>::contains_key(&current_session,&heartbeat.authority_index);

	}

	fn local_authority_keys() -> impl Iterator<Item=(u32, T::AuthorityId)> {
		// on-chain storage
		//
		// At index `idx`:
		// 1. A (ImOnline) public key to be used by a validator at index `idx` to send im-online
		//          heartbeats.
		let authorities = Keys::<T>::get();

		// local keystore
		//
		// All `ImOnline` public (+private) keys currently in the local keystore.
		let mut local_keys = T::AuthorityId::all();

		local_keys.sort();

		authorities.into_iter()
			.enumerate()
			.filter_map(move |(index, authority)| {
				local_keys.binary_search(&authority)
					.ok()
					.map(|location| (index as u32, local_keys[location].clone()))
			})
	}
}

impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		let valid_tx = |provide| ValidTransaction::with_tag_prefix("ocw-demo")
			.priority(UNSIGNED_TXS_PRIORITY)
			.and_provides([&provide])
			.longevity(3)
			.propagate(true)
			.build();

		match call {
			Call::submit_number_unsigned(_number) => valid_tx(b"submit_number_unsigned".to_vec()),
			Call::submit_number_unsigned_with_signed_payload(ref payload, ref signature) => {
				if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
					return InvalidTransaction::BadProof.into();
				}
				valid_tx(b"submit_number_unsigned_with_signed_payload".to_vec())
			},
			_ => InvalidTransaction::Call.into(),
		}
	}
}

impl<T: Trait> rt_offchain::storage_lock::BlockNumberProvider for Module<T> {
	type BlockNumber = T::BlockNumber;
	fn current_block_number() -> Self::BlockNumber {
	  <frame_system::Module<T>>::block_number()
	}
}
