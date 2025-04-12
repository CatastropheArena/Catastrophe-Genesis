// #[test_only]
// module nexus::card_interaction_tests {
//     use sui::test_scenario::{Self as ts, Scenario};
//     use sui::test_utils::assert_eq;
//     use sui::clock::{Self, Clock};
//     use std::string;
    
//     use nexus::card::{Self, Card};
//     use nexus::rental::{Self, RentalMarket};
//     use nexus::staking::{Self, StakingPool};
//     use nexus::fragment::{Self, FragmentStore};
//     use nexus::passport::{Self, State, Passport};

//     // Test constants
//     const ADMIN: address = @0xAD;
//     const USER1: address = @0x1;
//     const USER2: address = @0x2;

//     fun init_test_env(scenario: &mut Scenario) {
//         // 初始化状态
//         ts::next_tx(scenario, ADMIN);
//         {
//             passport::init_for_testing(ts::ctx(scenario));
//         };

//         // 初始化卡牌系统
//         ts::next_tx(scenario, ADMIN);
//         {
//             card::init_for_testing(ts::ctx(scenario));
//         };

//         // 初始化租赁市场
//         ts::next_tx(scenario, ADMIN);
//         {
//             rental::init_for_testing(ts::ctx(scenario));
//         };

//         // 初始化质押池
//         ts::next_tx(scenario, ADMIN);
//         {
//             staking::init_for_testing(ts::ctx(scenario));
//         };

//         // 初始化碎片商店
//         ts::next_tx(scenario, ADMIN);
//         {
//             fragment::init_for_testing(ts::ctx(scenario));
//         };
//     }

//     #[test]
//     fun test_card_synthesis() {
//         let scenario = ts::begin(ADMIN);
        
//         // 初始化测试环境
//         init_test_env(&mut scenario);
        
//         // 创建用户护照
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let state = ts::take_shared<State>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
//             let passport = passport::create_passport(&mut state, &clock, ts::ctx(&mut scenario));
//             passport::transfer_passport(passport, USER1);
//             clock::destroy_for_testing(clock);
//             ts::return_shared(state);
//         };

//         // 合成卡牌
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let passport = ts::take_from_sender<Passport>(&scenario);
//             let store = ts::take_shared<FragmentStore>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
//             // 假设用户已经有足够的碎片
//             let card = card::synthesize_card(
//                 &mut passport,
//                 &mut store,
//                 string::utf8(b"Synthesized Card"),
//                 1, // Uncommon rarity
//                 &clock,
//                 ts::ctx(&mut scenario)
//             );
            
//             card::transfer_card(card, USER1);
            
//             clock::destroy_for_testing(clock);
//             ts::return_to_sender(&scenario, passport);
//             ts::return_shared(store);
//         };

//         ts::end(scenario);
//     }

//     #[test]
//     fun test_card_rental() {
//         let scenario = ts::begin(ADMIN);
        
//         // 初始化测试环境
//         init_test_env(&mut scenario);
        
//         // 创建卡牌
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
//             let card = card::create_card(
//                 string::utf8(b"Rental Card"),
//                 1, // Uncommon rarity
//                 string::utf8(b"https://example.com/image.png"),
//                 ts::ctx(&mut scenario)
//             );
//             card::transfer_card(card, USER1);
//             clock::destroy_for_testing(clock);
//         };

//         // 出租卡牌
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let market = ts::take_shared<RentalMarket>(&scenario);
//             let card = ts::take_from_sender<Card>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
//             rental::list_card(
//                 &mut market,
//                 card,
//                 100, // 租金
//                 86400000, // 24小时租期
//                 &clock,
//                 ts::ctx(&mut scenario)
//             );
            
//             clock::destroy_for_testing(clock);
//             ts::return_shared(market);
//         };

//         // 租用卡牌
//         ts::next_tx(&mut scenario, USER2);
//         {
//             let market = ts::take_shared<RentalMarket>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
//             rental::rent_card(
//                 &mut market,
//                 0, // 第一个列表的卡牌
//                 &clock,
//                 ts::ctx(&mut scenario)
//             );
            
//             clock::destroy_for_testing(clock);
//             ts::return_shared(market);
//         };

//         ts::end(scenario);
//     }

//     #[test]
//     fun test_card_staking() {
//         let scenario = ts::begin(ADMIN);
        
//         // 初始化测试环境
//         init_test_env(&mut scenario);
        
//         // 创建卡牌
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
//             let card = card::create_card(
//                 string::utf8(b"Staking Card"),
//                 2, // Rare rarity
//                 string::utf8(b"https://example.com/image.png"),
//                 ts::ctx(&mut scenario)
//             );
//             card::transfer_card(card, USER1);
//             clock::destroy_for_testing(clock);
//         };

//         // 质押卡牌
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let pool = ts::take_shared<StakingPool>(&scenario);
//             let card = ts::take_from_sender<Card>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
//             staking::stake_card(
//                 &mut pool,
//                 card,
//                 &clock,
//                 ts::ctx(&mut scenario)
//             );
            
//             clock::destroy_for_testing(clock);
//             ts::return_shared(pool);
//         };

//         // 快进24小时
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let pool = ts::take_shared<StakingPool>(&scenario);
//             let mut clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
//             clock::increment_for_testing(&mut clock, 1000 * 60 * 60 * 24);
            
//             // 领取质押奖励
//             staking::claim_rewards(
//                 &mut pool,
//                 0, // 第一个质押位置
//                 &clock,
//                 ts::ctx(&mut scenario)
//             );
            
//             clock::destroy_for_testing(clock);
//             ts::return_shared(pool);
//         };

//         // 取消质押
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let pool = ts::take_shared<StakingPool>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
//             staking::unstake_card(
//                 &mut pool,
//                 0, // 第一个质押位置
//                 &clock,
//                 ts::ctx(&mut scenario)
//             );
            
//             clock::destroy_for_testing(clock);
//             ts::return_shared(pool);
//         };

//         ts::end(scenario);
//     }

//     #[test]
//     #[expected_failure(abort_code = card::EInsufficientFragments)]
//     fun test_synthesis_without_fragments() {
//         let scenario = ts::begin(ADMIN);
        
//         // 初始化测试环境
//         init_test_env(&mut scenario);
        
//         // 创建用户护照
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let state = ts::take_shared<State>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
//             let passport = passport::create_passport(&mut state, &clock, ts::ctx(&mut scenario));
//             passport::transfer_passport(passport, USER1);
//             clock::destroy_for_testing(clock);
//             ts::return_shared(state);
//         };

//         // 尝试在没有碎片的情况下合成卡牌
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let passport = ts::take_from_sender<Passport>(&scenario);
//             let store = ts::take_shared<FragmentStore>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
//             // 这里应该会失败
//             let card = card::synthesize_card(
//                 &mut passport,
//                 &mut store,
//                 string::utf8(b"Failed Synthesis Card"),
//                 1, // Uncommon rarity
//                 &clock,
//                 ts::ctx(&mut scenario)
//             );
            
//             card::transfer_card(card, USER1);
            
//             clock::destroy_for_testing(clock);
//             ts::return_to_sender(&scenario, passport);
//             ts::return_shared(store);
//         };

//         ts::end(scenario);
//     }

//     #[test]
//     #[expected_failure(abort_code = rental::EInvalidRentalPeriod)]
//     fun test_invalid_rental_period() {
//         let scenario = ts::begin(ADMIN);
        
//         // 初始化测试环境
//         init_test_env(&mut scenario);
        
//         // 创建卡牌
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
//             let card = card::create_card(
//                 string::utf8(b"Invalid Rental Card"),
//                 1,
//                 string::utf8(b"https://example.com/image.png"),
//                 ts::ctx(&mut scenario)
//             );
//             card::transfer_card(card, USER1);
//             clock::destroy_for_testing(clock);
//         };

//         // 尝试设置无效的租期
//         ts::next_tx(&mut scenario, USER1);
//         {
//             let market = ts::take_shared<RentalMarket>(&scenario);
//             let card = ts::take_from_sender<Card>(&scenario);
//             let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
//             // 这里应该会失败，因为租期太短
//             rental::list_card(
//                 &mut market,
//                 card,
//                 100,
//                 1000, // 太短的租期
//                 &clock,
//                 ts::ctx(&mut scenario)
//             );
            
//             clock::destroy_for_testing(clock);
//             ts::return_shared(market);
//         };

//         ts::end(scenario);
//     }
// } 