#[test_only]
module nexus::card_tests {
    use sui::test_scenario::{Self as ts, Scenario};
    use sui::test_utils::{assert_eq};
    use sui::object::{Self, ID};
    use std::string::{Self, String};
    use nexus::card::{Self, Card};

    // 测试常量
    const ADMIN: address = @0xA;
    const USER: address = @0xB;

    // 辅助函数：创建测试场景
    fun create_scenario(): Scenario {
        ts::begin(ADMIN)
    }

    // 测试创建卡牌
    #[test]
    fun test_create_card() {
        let scenario = create_scenario();
        let ctx = ts::ctx(&mut scenario);
        
        // 创建卡牌
        let card_name = string::utf8(b"Test Card");
        let rarity = 1; // Uncommon
        let image_url = string::utf8(b"https://example.com/image.png");
        
        let card = card::create_card(card_name, rarity, image_url, ctx);
        
        // 验证卡牌属性
        assert_eq(card::get_name(&card), card_name);
        assert_eq(card::get_rarity(&card), rarity);
        assert_eq(card::get_level(&card), 0);
        assert_eq(card::get_image_url(&card), image_url);
        assert_eq(card::get_owner(&card), ADMIN);
        
        // 清理
        ts::end(scenario);
        
        // 销毁卡牌对象
        let ctx = ts::ctx(&mut scenario);
        let (_, _) = card::burn_card(card, ctx);
    }

    // 测试升级卡牌
    #[test]
    fun test_upgrade_card() {
        let scenario = create_scenario();
        let ctx = ts::ctx(&mut scenario);
        
        // 创建卡牌
        let card_name = string::utf8(b"Test Card");
        let rarity = 1; // Uncommon
        let image_url = string::utf8(b"https://example.com/image.png");
        
        let mut card = card::create_card(card_name, rarity, image_url, ctx);
        
        // 升级卡牌
        let ctx = ts::ctx(&mut scenario);
        card::upgrade_card(&mut card, ctx);
        
        // 验证升级后的等级
        assert_eq(card::get_level(&card), 1);
        
        // 再次升级
        let ctx = ts::ctx(&mut scenario);
        card::upgrade_card(&mut card, ctx);
        assert_eq(card::get_level(&card), 2);
        
        // 清理
        ts::end(scenario);
        
        // 销毁卡牌对象
        let ctx = ts::ctx(&mut scenario);
        let (_, _) = card::burn_card(card, ctx);
    }

    // 测试销毁卡牌
    #[test]
    fun test_burn_card() {
        let scenario = create_scenario();
        let ctx = ts::ctx(&mut scenario);
        
        // 创建卡牌
        let card_name = string::utf8(b"Test Card");
        let rarity = 2; // Rare
        let image_url = string::utf8(b"https://example.com/image.png");
        
        let card = card::create_card(card_name, rarity, image_url, ctx);
        
        // 销毁卡牌
        let ctx = ts::ctx(&mut scenario);
        let (returned_rarity, owner) = card::burn_card(card, ctx);
        
        // 验证返回的信息
        assert_eq(returned_rarity, rarity);
        assert_eq(owner, ADMIN);
        
        // 清理
        ts::end(scenario);
    }
} 