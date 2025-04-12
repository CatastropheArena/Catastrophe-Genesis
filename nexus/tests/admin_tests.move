#[test_only]
module nexus::admin_tests {
    use sui::test_scenario as ts;
    use sui::clock::{Self};
    use sui::coin::{Self};
    use std::string;
    use nexus::admin::{Self, AdminCap};
    use nexus::treasury::{Self, Treasury};
    use nexus::fish::{FISH};


    // Test addresses
    const ADMIN: address = @0x1;
    const USER1: address = @0x42;

    #[test]
    fun test_admin_init() {
        let mut scenario = ts::begin(ADMIN);
        {
            admin::init_for_testing(ts::ctx(&mut scenario));
        };
        ts::next_tx(&mut scenario, ADMIN);
        {
            assert!(ts::has_most_recent_for_sender<AdminCap>(&scenario), 0);
            let admin_cap = ts::take_from_sender<AdminCap>(&scenario);
            ts::return_to_sender(&scenario, admin_cap);
        };
        ts::end(scenario);
    }

    #[test]
    fun test_admin_transfer() {
        let mut scenario = ts::begin(ADMIN);
        // 初始化管理员和资金库
        {
            admin::init_for_testing(ts::ctx(&mut scenario));
            treasury::init_for_testing(ts::ctx(&mut scenario));
        };
        
        // 为资金库添加初始资金
        ts::next_tx(&mut scenario, ADMIN);
        {
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let fish = coin::mint_for_testing<FISH>(10000, ts::ctx(&mut scenario));
            treasury::deposit(&mut treasury, fish, string::utf8(b"Initial funds"), &clock, ts::ctx(&mut scenario));
            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
        };

        // 转移管理员权限
        ts::next_tx(&mut scenario, ADMIN);
        {
            let admin_cap = ts::take_from_sender<AdminCap>(&scenario);
            let treasury = ts::take_shared<Treasury>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            // 转移管理员权限到 USER1
            admin::transfer_admin_cap(admin_cap, USER1, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
        };

        // 验证转移并使用新权限
        ts::next_tx(&mut scenario, USER1);
        {
            // 验证 USER1 现在拥有管理员权限
            let admin_cap = ts::take_from_sender<AdminCap>(&scenario);
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            // 尝试使用新管理员权限执行操作
            let coin = treasury::withdraw(&mut treasury, 1000, &admin_cap, ts::ctx(&mut scenario));
            transfer::public_transfer(coin, USER1);
            
            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
            ts::return_to_sender(&scenario, admin_cap);
        };
        ts::end(scenario);
    }

    #[test]
    #[expected_failure(abort_code = nexus::treasury::EInsufficientBalance)]
    fun test_admin_withdraw_insufficient_balance() {
        let mut scenario = ts::begin(ADMIN);
        {
            admin::init_for_testing(ts::ctx(&mut scenario));
            treasury::init_for_testing(ts::ctx(&mut scenario));
        };
        ts::next_tx(&mut scenario, ADMIN);
        {
            let admin_cap = ts::take_from_sender<AdminCap>(&scenario);
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            // 尝试提取超过余额的金额
            let coin = treasury::withdraw(&mut treasury, 1000, &admin_cap, ts::ctx(&mut scenario));
            transfer::public_transfer(coin, ADMIN);
            
            clock::destroy_for_testing(clock);
            ts::return_shared(treasury);
            ts::return_to_sender(&scenario, admin_cap);
        };
        ts::end(scenario);
    }
} 