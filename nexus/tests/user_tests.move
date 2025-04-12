#[test_only]
module nexus::user_tests {
    use sui::test_scenario::{Self as ts, Scenario};
    use sui::clock::{Self, Clock};
    use sui::test_utils::assert_eq;
    use sui::coin::{Self, Coin};
    use sui::transfer;
    use std::string;
    
    use nexus::user::{Self};
    use nexus::treasury::{Self, Treasury};
    use nexus::passport::{Self, State, Passport};
    use nexus::fragment::{Self, FragmentStore};
    use nexus::fish::{FISH};

    // Test constants
    const ADMIN: address = @0xAD;
    const USER1: address = @0x1;
    const USER2: address = @0x2;

    fun init_test_env(scenario: &mut Scenario) {
        // 初始化护照状态
        ts::next_tx(scenario, ADMIN);
        {
            passport::init_for_testing(ts::ctx(scenario));
        };

        // 初始化资金库
        ts::next_tx(scenario, ADMIN);
        {
            treasury::init_for_testing(ts::ctx(scenario));
        };

        // 为资金库添加初始资金
        ts::next_tx(scenario, ADMIN);
        {
            let mut treasury = ts::take_shared<Treasury>(scenario);
            let clock = clock::create_for_testing(ts::ctx(scenario));
            let fish = coin::mint_for_testing<FISH>(1000000, ts::ctx(scenario));
            treasury::deposit(
                &mut treasury, 
                fish,
                string::utf8(b"Initial treasury funds"),
                &clock,
                ts::ctx(scenario)
            );
            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
        };

        // 初始化碎片商店
        ts::next_tx(scenario, ADMIN);
        {
            fragment::init_for_testing(ts::ctx(scenario));
        };

        // 确认所有共享对象都已正确初始化
        ts::next_tx(scenario, ADMIN);
        {
            assert!(ts::has_most_recent_shared<State>(), 0);
            assert!(ts::has_most_recent_shared<Treasury>(), 1);
            assert!(ts::has_most_recent_shared<FragmentStore>(), 2);
        };
    }

    #[test]
    fun test_create_new_user() {
        let mut scenario = ts::begin(ADMIN);
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 创建新用户
        ts::next_tx(&mut scenario, USER1);
        {
            let mut state = ts::take_shared<State>(&scenario);
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            user::create_new_user(
                &mut state,
                &mut treasury,
                &mut store,
                &clock,
                ts::ctx(&mut scenario)
            );

            clock::destroy_for_testing(clock);
            ts::return_shared(state);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        // 验证用户创建成功
        ts::next_tx(&mut scenario, USER1);
        {
            assert!(ts::has_most_recent_for_address<Passport>(USER1), 0);
            let passport = ts::take_from_sender<Passport>(&scenario);
            ts::return_to_sender(&scenario, passport);
        };

        ts::end(scenario);
    }

    #[test]
    fun test_daily_rewards() {
        let mut scenario = ts::begin(ADMIN);
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 创建用户
        ts::next_tx(&mut scenario, USER1);
        {
            let mut state = ts::take_shared<State>(&scenario);
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            user::create_new_user(
                &mut state,
                &mut treasury,
                &mut store,
                &clock,
                ts::ctx(&mut scenario)
            );

            clock::destroy_for_testing(clock);
            ts::return_shared(state);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        // 快进24小时
        ts::next_tx(&mut scenario, USER1);
        {
            let mut passport = ts::take_from_sender<Passport>(&scenario);
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let mut clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            // 快进24小时
            clock::increment_for_testing(&mut clock, 86400000); // 24小时的毫秒数
            
            // 现在应该可以领取奖励了
            user::claim_daily_rewards(
                &mut passport,
                &mut treasury,
                &mut store,
                &clock,
                ts::ctx(&mut scenario)
            );

            clock::destroy_for_testing(clock);
            ts::return_to_sender(&scenario, passport);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        // 尝试在同一天再次领取（应该失败）
        ts::next_tx(&mut scenario, USER1);
        {
            let mut passport = ts::take_from_sender<Passport>(&scenario);
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));

            // 这里应该会失败，因为还没过24小时
            let mut failed = false;
            if (!passport::can_claim_daily_rewards(&passport, &clock)) {
                failed = true;
            };
            assert!(failed, 0);

            clock::destroy_for_testing(clock);
            ts::return_to_sender(&scenario, passport);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        ts::end(scenario);
    }

    #[test]
    fun test_buy_fragments() {
        let mut scenario = ts::begin(ADMIN);
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 创建用户并给予一些FISH代币
        ts::next_tx(&mut scenario, USER1);
        {
            let mut state = ts::take_shared<State>(&scenario);
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            user::create_new_user(
                &mut state,
                &mut treasury,
                &mut store,
                &clock,
                ts::ctx(&mut scenario)
            );

            // 创建一些FISH代币
            let fish = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            transfer::public_transfer(fish, USER1);

            clock::destroy_for_testing(clock);
            ts::return_shared(state);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        // 购买碎片
        ts::next_tx(&mut scenario, USER1);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let payment = coin::mint_for_testing<FISH>(100, ts::ctx(&mut scenario));
            
            user::buy_fragments(
                payment,
                50, // 购买50个碎片
                &mut store,
                &mut treasury,
                &clock,
                ts::ctx(&mut scenario)
            );

            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        ts::end(scenario);
    }

    #[test]
    #[expected_failure(abort_code = user::EIncorrectPaymentAmount)]
    fun test_buy_fragments_with_incorrect_amount() {
        let mut scenario = ts::begin(ADMIN);
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 尝试用错误的金额购买碎片
        ts::next_tx(&mut scenario, USER1);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let payment = coin::mint_for_testing<FISH>(90, ts::ctx(&mut scenario)); // 错误的金额
            
            user::buy_fragments(
                payment,
                50,
                &mut store,
                &mut treasury,
                &clock,
                ts::ctx(&mut scenario)
            );

            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        ts::end(scenario);
    }
} 