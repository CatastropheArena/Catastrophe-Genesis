#[test_only]
module nexus::admin_tests {
    use sui::test_scenario::{Self as ts, Scenario};
    use sui::test_utils::assert_eq;
    use sui::clock::{Self, Clock};
    use sui::coin::{Self};
    use std::string;
    
    use nexus::admin::{Self};
    use nexus::treasury::{Self, Treasury};
    use nexus::passport::{Self, State};
    use nexus::fragment::{Self, FragmentStore};
    use nexus::fish::{FISH};

    // Test constants
    const ADMIN: address = @0xAD;
    const USER1: address = @0x1;
    const USER2: address = @0x2;

    fun init_test_env(scenario: &mut Scenario) {
        // 初始化状态
        ts::next_tx(scenario, ADMIN);
        {
            passport::init_for_testing(ts::ctx(scenario));
        };

        // 初始化资金库
        ts::next_tx(scenario, ADMIN);
        {
            treasury::init_for_testing(ts::ctx(scenario));
        };

        // 初始化碎片商店
        ts::next_tx(scenario, ADMIN);
        {
            fragment::init_for_testing(ts::ctx(scenario));
        };
    }

    #[test]
    fun test_admin_initialization() {
        let mut scenario = ts::begin(ADMIN);
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 验证管理员权限
        ts::next_tx(&mut scenario, ADMIN);
        {
            let treasury = ts::take_shared<Treasury>(&scenario);
            assert!(treasury::get_admin(&treasury) == ADMIN, 0);
            ts::return_shared(treasury);
        };

        ts::end(scenario);
    }

    #[test]
    #[expected_failure(abort_code = treasury::ENotAuthorized)]
    fun test_unauthorized_admin_action() {
        let mut scenario = ts::begin(ADMIN);
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 非管理员尝试执行管理员操作
        ts::next_tx(&mut scenario, USER1);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            // 这里应该会失败，因为USER1不是管理员
            treasury::change_admin(
                &mut treasury,
                USER2,
                &clock,
                ts::ctx(&mut scenario)
            );
            
            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
        };

        ts::end(scenario);
    }

    #[test]
    fun test_admin_change() {
        let mut scenario = ts::begin(ADMIN);
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 管理员更换
        ts::next_tx(&mut scenario, ADMIN);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            treasury::change_admin(
                &mut treasury,
                USER1,
                &clock,
                ts::ctx(&mut scenario)
            );
            
            assert!(treasury::get_admin(&treasury) == USER1, 0);
            
            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
        };

        ts::end(scenario);
    }

    #[test]
    fun test_admin_withdraw() {
        let mut scenario = ts::begin(ADMIN);
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 存入一些资金
        ts::next_tx(&mut scenario, USER1);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            
            treasury::deposit(
                &mut treasury,
                payment,
                string::utf8(b"Test deposit"),
                &clock,
                ts::ctx(&mut scenario)
            );
            
            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
        };

        // 管理员提取资金
        ts::next_tx(&mut scenario, ADMIN);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let coin = treasury::withdraw(&mut treasury, 500, ts::ctx(&mut scenario));
            assert!(coin::value(&coin) == 500, 0);
            coin::burn_for_testing(coin);
            ts::return_shared(treasury);
        };

        ts::end(scenario);
    }
} 