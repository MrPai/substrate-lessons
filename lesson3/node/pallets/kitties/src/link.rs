use frame_support::{StorageMap, Parameter};
use sp_runtime::traits::Member;
use codec::{Encode, Decode};

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode, Clone)]
pub struct LinkedItem<Value> {
	pub prev: Option<Value>,
	pub next: Option<Value>,
}

pub struct LinkedList<Storage, Key, Value>(sp_std::marker::PhantomData<(Storage, Key, Value)>);

impl<Storage, Key, Value> LinkedList<Storage, Key, Value> where
	Value: Parameter + Member + Copy,
	Key: Parameter,
	Storage: StorageMap<(Key, Option<Value>), LinkedItem<Value>, Query = Option<LinkedItem<Value>>>,
{
	fn read_head(key: &Key) -> LinkedItem<Value> {
		Self::read(key, None)
	}

	fn write_head(account: &Key, item: LinkedItem<Value>) {
		Self::write(account, None, item);
	}

	fn read(key: &Key, value: Option<Value>) -> LinkedItem<Value> {
		Storage::get((&key, value)).unwrap_or_else(|| LinkedItem {
			prev: None,
			next: None,
		})
	}

	fn write(key: &Key, value: Option<Value>, item: LinkedItem<Value>) {
		Storage::insert((&key, value), item);
	}

	//append只会往head追加最新值
	//key作为account id，一个用户是唯一的，
	//首先判断(key、value)是否存在，如果存在，不进行任何操作；
	//如果不存在，插入到head
	pub fn append(key: &Key, value: Value) -> bool {
		// 作业
		let if_item_exist = Self::read(key, Some(value));
		let if_head_exist = Self::read_head(key);
		if None == if_item_exist.prev && None == if_head_exist.prev {
			let current_head = Self::read_head(key);
			//如果为None，说明该key下没有任何元素
			//执行else分支，插入该key下的第一个头元素
			if let Some(next) = current_head.next {
				//最新头元素的下一个元素，这一个元素是之前的头元素
				let next_item = LinkedItem {
					prev: Some(value), //插入最新头元素后，当前头元素的prev指向最新头元素的kittyindex
					next: Some(next),//当前头元素的next不会被最新头元素影响
				};
				//对于最初插入的第一个头元素，prev == next == itself
				//除最初第一个头元素之外的当前头元素，prev == itself
				let prev =current_head.prev.unwrap();
				Self::write(key, Some(prev), next_item);
				let current_head = LinkedItem {
					prev: Some(value),//最新头元素的 prev == itself
					next: Some(prev),
				};
				Self::write_head(key, current_head);
			} else {
				let item = LinkedItem {
					prev: Some(value),
					next: Some(value),
				};
				Self::write_head(key, item);
			}
			return true
		} else {
			//元素已经存在，不做任何操作
			return false
		}
	}

	//根据key可以remove任意item
	pub fn remove(key: &Key, value: Value) -> bool {
		// 作业
		let if_item = Self::read(key, Some(value));
		if let Some(prev) = if_item.prev {
			//执行删除操作
			let if_item = LinkedItem {
				prev: None,
				next: None,
			};
			Self::write(key, Some(value), if_item.clone());

			let next = if_item.next.unwrap();
			if next == value {
				//获取倒数第二个元素,可能是中间元素也可能是头元素，不可能不存在（因为if_item能读取出来）
				/////删除的是当前尾元素
				let last_but_one =  Self::read(key, Some(prev));
				if None == last_but_one.prev {
					//倒数第二个元素为头元素，必定为头元素
					let last_but_one_head =  Self::read_head(key);
					let new_head = LinkedItem {
						prev: last_but_one_head.prev,//头元素的prev就是它自己
						next: last_but_one_head.prev,//删除后只剩下唯一一个头元素了
					};
					Self::write_head(key, new_head);
				} else {
					//倒数第二个元素为中间元素
					//删除尾元素后，倒数第二个元素变为新的尾元素
					let new_tail = LinkedItem {
						prev: last_but_one.prev,//prev不变
						next: Some(prev),//代表第二个元素自己的index
					};
					Self::write(key, Some(prev), new_tail);
				}
			} else {
				/////删除是当前中间元素
				//把上一个元素和下一个元素衔接起来即可
				let (if_head, prev_item) = {
					//上一个元素有可能是中间元素，也可能是头元素
					let prev_item = Self::read(key, if_item.prev);
					let prev_head = Self::read_head(key);
					if if_item.prev == prev_head.prev {
						(true, prev_head)
					} else {
						(false, prev_item)
					}
				};
				//不需要理会下一个元素是尾元素还是中间元素
				let next_item = Self::read(key, if_item.next);
				let new_prev_item = LinkedItem {
					prev: prev_item.prev,
					next: if_item.next,
				};
				if if_head {
					Self::write_head(key, new_prev_item);
				} else {
					Self::write(key,if_item.prev, new_prev_item);
				}
				let new_next_item = LinkedItem {
					prev: if_item.prev,
					next: next_item.next,
				};
				Self::write(key,if_item.next, new_next_item);
			}
			return true;
		} else {
			let if_head = Self::read_head(key);
			if let Some(prev) = if_head.prev {
				if prev == value {
					//删除当前头元素
					let second_item = Self::read(key, if_head.next);
					let new_head = LinkedItem {
						prev: if_item.next,
						next: second_item.next,
					};
					//当前第二个元素成为新的头元素
					//覆盖需要删除的当前头元素
					Self::write_head(key, new_head);
					return true
				} else {
					//当前元素不存在
					return false
				}
			} else {
				//当前没有任何元素
				return false
			}
		}
	}
}
