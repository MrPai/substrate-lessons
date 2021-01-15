use crate::{Error, mock::{Event as mockEvent,*}};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_kitties_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_eq!(KittiesModule::create(Origin::signed(1),10,10), Ok(()));
		assert_eq!(last_event(),mockEvent::kitties(RawEvent::Created(1, 0, 10, 10)));
		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::create(Origin::signed(1),50,12), Ok(()));
		assert_eq!(KittiesModule::kitties_count(), 2);
		let kitty0: Kitty<Test> = Kitties::<Test>::get(0).unwrap();
		assert!(kitty0.dna.iter().map(|&i| i as u32).sum::<u32>() > 0);
		assert_eq!(kitty0.generation, 0);
		assert_eq!(kitty0.matron_id, None);
		assert_eq!(kitty0.sire_id, None);
		assert_eq!(KittyOwners::<Test>::get(0), Some(1));
		assert_eq!(<OwnerKitties<Test>>::iter_prefix_values(1).count(), 2);	
		let mut v  = <OwnerKitties<Test>>::iter_prefix_values(1).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [0,1]);

		assert_eq!(
			events(),
			[
				mockEvent::kitties(RawEvent::Created(1, 0, 10, 10)),
				mockEvent::kitties(RawEvent::Created(1, 1, 50, 12)),
			]
		);
	});
}

#[test]
fn create_kitties_then_get_kitties_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_eq!(KittiesModule::create(Origin::signed(1),10,10), Ok(()));
		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::create(Origin::signed(1),10,10), Ok(()));
		assert_eq!(KittiesModule::kitties_count(), 2);
		let kitty0: Kitty<Test> = Kitties::<Test>::get(0).unwrap();
		assert!(kitty0.dna.iter().map(|&i| i as u32).sum::<u32>() > 0);
		assert_eq!(kitty0.generation, 0);
		assert_eq!(kitty0.matron_id, None);
		assert_eq!(kitty0.sire_id, None);
		let kitty1: Kitty<Test> = Kitties::<Test>::get(1).unwrap();
		assert!(kitty1.dna.iter().map(|&i| i as u32).sum::<u32>() > 0);
		assert_eq!(kitty1.generation, 0);
		assert_eq!(kitty1.matron_id, None);
		assert_eq!(kitty1.sire_id, None);
	});
}

#[test]
fn create_kitties_failed_when_reach_the_max() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		<KittiesCount<Test>>::put(u32::MAX);
		assert_noop!(
			KittiesModule::create(Origin::signed(1),10,10),
			Error::<Test>::KittiesCountOverflow
		);
	});
}

#[test]
fn create_kitties_failed_when_not_enough_deposit() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_noop!(
			KittiesModule::create(Origin::signed(1),10,9),
			Error::<Test>::NotEnoughDeposit
		);
	});
}


#[test]
fn get_one_account_all_kitties_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_eq!(KittiesModule::create(Origin::signed(1),10,10), Ok(()));
		assert_eq!(KittiesModule::create(Origin::signed(1),10,10), Ok(()));
		assert_eq!(KittiesModule::create(Origin::signed(1),10,10), Ok(()));
		assert_eq!(<OwnerKitties<Test>>::iter_prefix_values(1).count(), 3);	
		let mut v  = <OwnerKitties<Test>>::iter_prefix_values(1).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [0,1,2]);
		let owner1_kitties_vec = v.iter().map(|&i: &u32|{
			let kitty: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			kitty
		}).collect::<Vec<Kitty<Test>>>();
		assert_eq!(owner1_kitties_vec.len(), 3);

		assert_eq!(KittiesModule::create(Origin::signed(2),10,10), Ok(()));
		assert_eq!(KittiesModule::create(Origin::signed(2),10,10), Ok(()));
		assert_eq!(<OwnerKitties<Test>>::iter_prefix_values(2).count(), 2);	
		let mut v  = <OwnerKitties<Test>>::iter_prefix_values(2).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [3,4]);
		let owner2_kitties_vec = v.iter().map(|&i: &u32|{
			let kitty: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			kitty
		}).collect::<Vec<Kitty<Test>>>();
		assert_eq!(owner2_kitties_vec.len(), 2);
	});
}

#[test]
fn breed_kitties_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		//kitty index is 0
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		//kitty index is 1
		assert_ok!(KittiesModule::create(Origin::signed(1),9,12));
		//kitty index is 2
		assert_ok!(KittiesModule::breed(Origin::signed(2),0,1,8,15));
		//current kitty index is 3
		assert_eq!(KittiesModule::kitties_count(),3);

		//fixme 任何账户都可以使用别人的kitty breed，理论上应该要限制
		assert_eq!(last_event(),mockEvent::kitties(RawEvent::Created(2, 2, 8, 15)));
		assert_eq!(
			events(),
			[
				mockEvent::kitties(RawEvent::Created(1, 0, 10, 10)),
				mockEvent::kitties(RawEvent::Created(1, 1, 9, 12)),
				mockEvent::kitties(RawEvent::Created(2, 2, 8, 15)),
			]
		);
	});
}

#[test]
fn breed_kitties_and_then_get_parents_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));

		//child kitty index is 2,matron index is 0, sire index is 1
		//sire generation is 0, so this child's generation is 1
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_eq!(KittiesModule::kitties_count() , 3);
		let child_kitty1: Kitty<Test> = Kitties::<Test>::get(2).unwrap();
		assert_eq!(child_kitty1.generation, 1);
		assert_eq!(child_kitty1.matron_id, Some(0));
		assert_eq!(child_kitty1.sire_id, Some(1));

		//child kitty index is 3,matron index is 1, sire index is 2
		//sire generation is 1, so this child's generation is 2
		assert_ok!(KittiesModule::breed(Origin::signed(1),1,2,10,10));
		let child_kitty1: Kitty<Test> = Kitties::<Test>::get(3).unwrap();
		assert_eq!(child_kitty1.generation, 2);
		assert_eq!(child_kitty1.matron_id, Some(1));
		assert_eq!(child_kitty1.sire_id, Some(2));
	});
}

#[test]
fn breed_kitties_and_then_get_brothers_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_eq!(<KittiesGeneration<Test>>::iter_prefix_values(0).count(), 2);
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_eq!(KittiesModule::kitties_count() , 7);
		assert_eq!(<KittiesGeneration<Test>>::iter_prefix_values(0).count(), 5);

		//测试两个零代猫breed的children
		let child_kitty6: Kitty<Test> = Kitties::<Test>::get(6).unwrap();
		assert_eq!(child_kitty6.generation, 1);
		assert_eq!(child_kitty6.matron_id, Some(0));
		assert_eq!(child_kitty6.sire_id, Some(1));
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(child_kitty6.matron_id.unwrap()).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [2,3,4,5,6]);
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(child_kitty6.sire_id.unwrap()).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [2,3,4,5,6]);

		//测试两个一代猫breed的children
		assert_ok!(KittiesModule::breed(Origin::signed(1),3,4,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),3,4,10,10));
		assert_eq!(KittiesModule::kitties_count() , 9);
		let child_kitty7: Kitty<Test> = Kitties::<Test>::get(7).unwrap();
		assert_eq!(child_kitty7.generation, 2);
		assert_eq!(child_kitty7.matron_id, Some(3));
		assert_eq!(child_kitty7.sire_id, Some(4));
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(child_kitty7.matron_id.unwrap()).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [7,8]);

		//测试一个零代猫和一个二代猫breed的children
		assert_ok!(KittiesModule::breed(Origin::signed(1),1,7,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),7,1,10,10));
		assert_eq!(KittiesModule::kitties_count() , 11);
		let child_kitty9: Kitty<Test> = Kitties::<Test>::get(9).unwrap();
		let child_kitty10: Kitty<Test> = Kitties::<Test>::get(10).unwrap();
		assert_eq!(child_kitty9.generation, 3);
		assert_eq!(child_kitty10.generation, 1);
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(child_kitty9.matron_id.unwrap()).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [2,3,4,5,6,9,10]);
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(child_kitty9.sire_id.unwrap()).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [9,10]);

		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(child_kitty10.matron_id.unwrap()).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [9,10]);
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(child_kitty10.sire_id.unwrap()).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [2,3,4,5,6,9,10]);
	});
}

#[test]
fn breed_kitties_and_then_get_children_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_eq!(KittiesModule::kitties_count() , 7);
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(0).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [2,3,4,5,6]);
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(1).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [2,3,4,5,6]);
	});
}

#[test]
fn breed_kitties_and_then_get_lover_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,1,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,2,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,3,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,4,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),0,5,10,10));
		assert_eq!(KittiesModule::kitties_count() , 7);
		let mut v  = <KittiesGeneration<Test>>::iter_prefix_values(0).collect::<Vec<u32>>();

		///////////////////////获取kitty0的lover
		let mut sire_ids = v.iter().map(|&i: &u32| {
			let child: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			//0 is matron_id，so its lover is sire_id
			child.sire_id.unwrap()
		}).collect::<Vec<u32>>();
		sire_ids.sort();
		assert_eq!(sire_ids, [1,2,3,4,5]);
		v.clear();
		sire_ids.clear();
		
		///////////////////////获取kitty1的lover
		v  = <KittiesGeneration<Test>>::iter_prefix_values(1).collect::<Vec<u32>>();
		let mut matron_ids = v.iter().map(|&i: &u32| {
			let child: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			//1 is sire_id，so its lover is matron_id
			child.matron_id.unwrap()
		}).collect::<Vec<u32>>();
		matron_ids.sort();
		assert_eq!(matron_ids, [0]);
		v.clear();
		matron_ids.clear();

		///////////////////////获取kitty2的lover
		v  = <KittiesGeneration<Test>>::iter_prefix_values(2).collect::<Vec<u32>>();
		let mut matron_ids = v.iter().map(|&i: &u32| {
			let child: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			//1 is sire_id，so its lover is matron_id
			child.matron_id.unwrap()
		}).collect::<Vec<u32>>();
		matron_ids.sort();
		assert_eq!(matron_ids, [0]);
		v.clear();
		matron_ids.clear();

		//breed again
		assert_ok!(KittiesModule::breed(Origin::signed(1),3,4,10,10));
		assert_ok!(KittiesModule::breed(Origin::signed(1),3,5,10,10));
		assert_eq!(KittiesModule::kitties_count() , 9);

		///////////////////////获取kitty3的lover
		let k= 3;
		v  = <KittiesGeneration<Test>>::iter_prefix_values(k).collect::<Vec<u32>>();
		v.sort();
		assert_eq!(v, [4, 7, 8]);//kitty0和kitty3 breed kitty4,，此时kitty3作为sire_id
		//获取3的sire id lover
		//注意去除掉自己，因为自己的children中，也包含了和其他matron_id breed的孩子
		sire_ids = v.iter().map(|&i: &u32| {
			let child: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			//3 is matron_id ，so its lover is sire_id
			child.sire_id.unwrap()
		}).filter(|&i| i!=k).collect::<Vec<u32>>();
		sire_ids.sort();
		assert_eq!(sire_ids, [4, 5]);

		//获取3的matron id lover
		//注意去除掉自己，因为自己的children中，也包含了和其他sire_id breed的孩子
		matron_ids = v.iter().map(|&i: &u32| {
			let child: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			//3 is sire_id，so its lover is matron_id
			child.matron_id.unwrap()
		}).filter(|&i| i!=k).collect::<Vec<u32>>();
		matron_ids.sort();
		assert_eq!(matron_ids, [0]);
		//获取3的lover，matron and sire
		matron_ids.append(&mut sire_ids);
		matron_ids.sort();
		assert_eq!(matron_ids, [0, 4, 5]);
		v.clear();
		matron_ids.clear();

		///////////////////////获取kitty4的lover
		let k= 4;
		v  = <KittiesGeneration<Test>>::iter_prefix_values(k).collect::<Vec<u32>>();
		v.sort();
		//kitty0&kitty4 breed kitty5
		//kitty3&kitty4 breed kitty7
		assert_eq!(v, [5, 7]);
		//获取4的sire id lover
		sire_ids = v.iter().map(|&i: &u32| {
			let child: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			//3 is matron_id ，so its lover is sire_id
			child.sire_id.unwrap()
		}).filter(|&i| i!=k).collect::<Vec<u32>>();
		assert_eq!(sire_ids.len(), 0);
		//获取4的matron id lover
		matron_ids = v.iter().map(|&i: &u32| {
			let child: Kitty<Test> = Kitties::<Test>::get(i).unwrap();
			//3 is sire_id，so its lover is matron_id
			child.matron_id.unwrap()
		}).filter(|&i| i!=k).collect::<Vec<u32>>();
		matron_ids.sort();
		assert_eq!(matron_ids, [0, 3]);
		//获取4的lover，matron and sire
		matron_ids.append(&mut sire_ids);
		matron_ids.sort();
		assert_eq!(matron_ids, [0, 3]);
		v.clear();
		matron_ids.clear();
	});
}

#[test]
fn breed_kitties_failed_when_no_parents_kitty() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),1,10));
		assert_ok!(KittiesModule::create(Origin::signed(1),1,10));
		assert_eq!(KittiesModule::kitties_count(), 2);
		assert_noop!(
			KittiesModule::breed(Origin::signed(1),2,0,10,10),
			Error::<Test>::InvalidMatronId
		);
		assert_noop!(
			KittiesModule::breed(Origin::signed(1),0,2,10,10),
			Error::<Test>::InvalidSireId
		);
		
	});
}

#[test]
fn breed_kitties_failed_when_same_parents_kitty() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_noop!(
			KittiesModule::breed(Origin::signed(1),0,0,10,10),
			Error::<Test>::RequireDifferentParents
		);
		
	});
}

#[test]
fn breed_kitties_failed_when_reach_the_max() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		<KittiesCount<Test>>::put(u32::MAX);
		assert_noop!(
			KittiesModule::breed(Origin::signed(1),0,1,10,10),
			Error::<Test>::KittiesCountOverflow
		);
		
	});
}

#[test]
fn breed_kitties_failed_when_not_enough_deposit() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_noop!(
			KittiesModule::breed(Origin::signed(1),0,1,10,9),
			Error::<Test>::NotEnoughDeposit
		);
	});
}


#[test]
fn transfer_kitties_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,11));
		assert_ok!(KittiesModule::transfer(Origin::signed(1),2,0,10));
		assert_eq!(last_event(),mockEvent::kitties(RawEvent::Transferred(1, 2, 0, 10)));
		assert_eq!(
			events(),
			[
				mockEvent::kitties(RawEvent::Created(1, 0, 10, 11)),
				mockEvent::kitties(RawEvent::Transferred(1, 2, 0, 10)),
			]
		);
	});
}

#[test]
fn transfer_kitties_failed_when_not_kitties_owner() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_ok!(KittiesModule::create(Origin::signed(2),10,10));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(2),1,0,10),
			Error::<Test>::NotKittiesOwner
		);
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1),2,1,10),
			Error::<Test>::NotKittiesOwner
		);
		assert_ok!(KittiesModule::transfer(Origin::signed(1),2,0,10));
		assert_ok!(KittiesModule::transfer(Origin::signed(2),1,0,10));
		
	});
}

#[test]
fn transfer_kitties_failed_when_transfer_self() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1),1,0,10),
			Error::<Test>::TransferSelf
		);
	});
}

#[test]
fn transfer_kitties_failed_when_not_enough_deposit() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,10));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1),2,0,9),
			Error::<Test>::NotEnoughTransferDeposit
		);
	});
}

#[test]
fn transfer_kitties_failed_when_not_enough_reserve() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,100));
		let sender_account = Account::<Test>::get(1).unwrap();
		assert_eq!(sender_account.reserved, 100);

		let sender_account = AccountData {
			free: 100,
			reserved: 9,
		};
		Account::<Test>::insert(1, sender_account);
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1),2,0,10),
			Error::<Test>::NotEnoughReservedDeposit
		);
	});
}

#[test]
fn transfer_kitties_failed_when_not_enough_for_transfer() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),10,50));
		let sender_account = Account::<Test>::get(1).unwrap();
		assert_eq!(sender_account.reserved, 50);
		Account::<Test>::insert(1, sender_account);
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1),2,0,51),
			Error::<Test>::NotEnoughForTransfer
		);
	});
}


#[test]
fn transfer_kitties_failed_when_no_kitties() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),1,10));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1),2,1,10),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn transfer_kitties_failed_when_invalid_account() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(KittiesModule::create(Origin::signed(1),1,10));
		assert_noop!(
			KittiesModule::transfer(Origin::signed(2),1,1,10),
			Error::<Test>::InvalidAccount
		);
	});
}
