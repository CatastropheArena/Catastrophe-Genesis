#[test_only]
#[allow(unused_variable, unused_function, unused_use, unused_let_mut)]
module nexus::card_tests {
    use sui::test_scenario::{Self as ts, Scenario};
    use sui::test_utils::{assert_eq};
    use sui::clock::{Self, Clock};
    use sui::random::{Self, Random};
    use sui::coin::{Self};
    use std::string::{Self, String};
    use nexus::card::{Self, Card};
    use nexus::fragment::{Self, FragmentStore, FRAGMENT};
    use nexus::treasury::{Self, Treasury};
    use nexus::fish::{FISH};

    // 测试常量
    const ADMIN: address = @0x0; // 系统地址
    const USER: address = @0xB;

    // 错误码
    const EINVALID_CARD_LEVEL: u64 = 1;
    const EINVALID_CARD_OWNER: u64 = 2;
    const EINVALID_CARD_RARITY: u64 = 3;

    fun init_test_env(scenario: &mut Scenario) {
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

        // 初始化随机数生成器
        ts::next_tx(scenario, ADMIN);
        {
            random::create_for_testing(ts::ctx(scenario));
        };
    }

    // 辅助函数：创建测试场景
    fun create_scenario(): Scenario {
        ts::begin(ADMIN)
    }

    // 辅助函数：创建一张卡牌
    fun create_card(scenario: &mut Scenario): Card {
        let mut treasury = ts::take_shared<Treasury>(scenario);
        let mut store = ts::take_shared<FragmentStore>(scenario);
        let clock = clock::create_for_testing(ts::ctx(scenario));
        let r = ts::take_shared<Random>(scenario);
        let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(scenario));
        
        card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(scenario));
        
        let card = ts::take_from_sender<Card>(scenario);
        
        clock::destroy_for_testing(clock);
        ts::return_shared(r);
        ts::return_shared(treasury);
        ts::return_shared(store);

        card
    }

    // 测试创建卡牌
    #[test]
    fun test_create_card() {
        let mut scenario = create_scenario();
        init_test_env(&mut scenario);
        
        // 抽取卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            
            card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(treasury);
        };
        
        // 验证卡牌属性
        ts::next_tx(&mut scenario, USER);
        {
            let card = ts::take_from_sender<Card>(&scenario);
            assert!(card::get_level(&card) == 0, EINVALID_CARD_LEVEL);
            assert!(card::get_owner(&card) == USER, EINVALID_CARD_OWNER);
            ts::return_to_sender(&scenario, card);
        };
        
        ts::end(scenario);
    }

    // 测试卡牌稀有度分布
    #[test]
    fun test_card_rarity_distribution() {
        let mut scenario = create_scenario();
        init_test_env(&mut scenario);
        
        let mut common_count = 0;
        let mut uncommon_count = 0;
        let mut rare_count = 0;
        let mut legendary_count = 0;
        let total_draws = 100;
        
        // 抽取100张卡牌并统计稀有度分布
        let mut i = 0;
        while (i < total_draws) {
            // 创建卡牌
            ts::next_tx(&mut scenario, USER);
            {
                let mut treasury = ts::take_shared<Treasury>(&scenario);
                let mut store = ts::take_shared<FragmentStore>(&scenario);
                let clock = clock::create_for_testing(ts::ctx(&mut scenario));
                let r = ts::take_shared<Random>(&scenario);
                let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
                
                card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(&mut scenario));
                
                clock::destroy_for_testing(clock);
                ts::return_shared(r);
                ts::return_shared(treasury);
                ts::return_shared(store);
            };
            
            // 验证卡牌
            ts::next_tx(&mut scenario, USER);
            {
                let card = ts::take_from_sender<Card>(&scenario);
                let rarity = card::get_rarity(&card);
                if (rarity == 70) { // RARITY_COMMON
                    common_count = common_count + 1;
                } else if (rarity == 20) { // RARITY_UNCOMMON
                    uncommon_count = uncommon_count + 1;
                } else if (rarity == 9) { // RARITY_RARE
                    rare_count = rare_count + 1;
                } else if (rarity == 1) { // RARITY_LEGENDARY
                    legendary_count = legendary_count + 1;
                };
                ts::return_to_sender(&scenario, card);
            };
            i = i + 1;
        };
        
        // 验证稀有度分布是否在合理范围内
        assert!(common_count > 50, 0); // 普通卡至少50%
        assert!(uncommon_count > 10, 0); // 非普通卡至少10%
        assert!(rare_count > 0, 0); // 稀有卡至少有1张
        assert!(legendary_count <= 5, 0); // 传说卡不超过5%
        
        ts::end(scenario);
    }

    // 测试卡牌合成
    #[test]
    fun test_combine_cards() {
        let mut scenario = create_scenario();
        init_test_env(&mut scenario);
        
        // 创建第一张卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            
            card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        // 创建第二张卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            
            card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };

        // 创建第三张卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            
            card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };
        
        // 合成卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let card1 = ts::take_from_sender<Card>(&scenario);
            let card2 = ts::take_from_sender<Card>(&scenario);
            let card3 = ts::take_from_sender<Card>(&scenario);
            
            // 记录原始卡牌的属性
            let total_level = card::get_level(&card1) + card::get_level(&card2) + card::get_level(&card3);
            let total_rarity = card::get_rarity(&card1) + card::get_rarity(&card2) + card::get_rarity(&card3);
            
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            
            // 铸造合成所需的碎片
            let fragments = fragment::mint(&mut store, 20, &clock, ts::ctx(&mut scenario));
            
            // 合成卡牌
            card::combine_card(card1, card2, card3, fragments, &mut store, &r, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(store);
        };

        // 验证新卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let new_card = ts::take_from_sender<Card>(&scenario);
            let new_level = card::get_level(&new_card);
            let new_rarity = card::get_rarity(&new_card);
            
            // 验证新卡牌的属性是否合理
            assert!(new_level <= 3, 0); // 新卡牌等级不应超过3（因为合成了3张0级卡）
            assert!(new_rarity <= 70, 0); // 新卡牌稀有度不应超过普通卡的稀有度
            assert!(card::get_owner(&new_card) == USER, EINVALID_CARD_OWNER);
            
            ts::return_to_sender(&scenario, new_card);
        };
        
        ts::end(scenario);
    }

    // 测试升级卡牌
    #[test]
    fun test_upgrade_card() {
        let mut scenario = create_scenario();
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 创建卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            
            card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };
        
        // 第一次升级尝试 (0级到1级，需要7个碎片，成功率80%)
        ts::next_tx(&mut scenario, USER);
        {
            let mut card = ts::take_from_sender<Card>(&scenario);
            let old_level = card::get_level(&card);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            
            let fragments = fragment::mint(&mut store, 7, &clock, ts::ctx(&mut scenario));
            card::upgrade_card(&mut card, fragments, &mut store, &r, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(store);
            ts::return_to_sender(&scenario, card);
        };

        // 验证第一次升级结果
        ts::next_tx(&mut scenario, USER);
        {
            let card = ts::take_from_sender<Card>(&scenario);
            let level = card::get_level(&card);
            assert!(level == 0 || level == 1, 0); // 等级应该是0或1
            ts::return_to_sender(&scenario, card);
        };
        
        // 第二次升级尝试
        ts::next_tx(&mut scenario, USER);
        {
            let mut card = ts::take_from_sender<Card>(&scenario);
            let old_level = card::get_level(&card);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            
            // 根据当前等级选择正确的碎片数量
            let fragment_amount = if (old_level == 0) { 7 } else { 15 };
            let fragments = fragment::mint(&mut store, fragment_amount, &clock, ts::ctx(&mut scenario));
            
            card::upgrade_card(&mut card, fragments, &mut store, &r, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(store);
            ts::return_to_sender(&scenario, card);
        };

        // 验证第二次升级结果
        ts::next_tx(&mut scenario, USER);
        {
            let card = ts::take_from_sender<Card>(&scenario);
            let level = card::get_level(&card);
            assert!(level <= 2, 0); // 等级不应超过2
            ts::return_to_sender(&scenario, card);
        };

        ts::end(scenario);
    }

    // 测试销毁卡牌
    #[test]
    fun test_burn_card() {
        let mut scenario = create_scenario();
        
        // 初始化测试环境
        init_test_env(&mut scenario);
        
        // 创建卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            
            card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };
        
        // 验证卡牌存在
        ts::next_tx(&mut scenario, USER);
        {
            let card = ts::take_from_sender<Card>(&scenario);
            assert!(card::get_owner(&card) == USER, EINVALID_CARD_OWNER);
            ts::return_to_sender(&scenario, card);
        };
        
        // 销毁卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let card = ts::take_from_sender<Card>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            
            card::burn_card(card, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
        };
        
        // 验证卡牌已被销毁
        ts::next_tx(&mut scenario, USER);
        {
            assert!(!ts::has_most_recent_for_sender<Card>(&scenario), 0);
        };
        
        ts::end(scenario);
    }

    // 测试卡牌属性
    #[test]
    fun test_card_attributes() {
        let mut scenario = create_scenario();
        init_test_env(&mut scenario);
        
        // 创建卡牌
        ts::next_tx(&mut scenario, USER);
        {
            let mut treasury = ts::take_shared<Treasury>(&scenario);
            let mut store = ts::take_shared<FragmentStore>(&scenario);
            let clock = clock::create_for_testing(ts::ctx(&mut scenario));
            let r = ts::take_shared<Random>(&scenario);
            let payment = coin::mint_for_testing<FISH>(1000, ts::ctx(&mut scenario));
            
            card::draw_card(&r, payment, &mut treasury, &clock, ts::ctx(&mut scenario));
            
            clock::destroy_for_testing(clock);
            ts::return_shared(r);
            ts::return_shared(treasury);
            ts::return_shared(store);
        };
        
        // 验证卡牌属性
        ts::next_tx(&mut scenario, USER);
        {
            let card = ts::take_from_sender<Card>(&scenario);
            
            // 验证基本属性
            assert!(card::get_level(&card) == 0, EINVALID_CARD_LEVEL);
            assert!(card::get_owner(&card) == USER, EINVALID_CARD_OWNER);
            
            // 验证稀有度是否在有效范围内
            let rarity = card::get_rarity(&card);
            assert!(
                rarity == 70 || // RARITY_COMMON
                rarity == 20 || // RARITY_UNCOMMON
                rarity == 9 ||  // RARITY_RARE
                rarity == 1,    // RARITY_LEGENDARY
                EINVALID_CARD_RARITY
            );
            
            // 验证卡牌名称不为空
            let name = card::get_name(&card);
            assert!(!string::is_empty(&name), 0);
            
            // 验证图片URL不为空
            let image_url = card::get_image_url(&card);
            assert!(!string::is_empty(&image_url), 0);
            
            ts::return_to_sender(&scenario, card);
        };
        
        ts::end(scenario);
    }
} 