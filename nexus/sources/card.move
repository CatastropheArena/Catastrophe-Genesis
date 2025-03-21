module nexus::card {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;
    use std::string::{Self, String};
    use std::vector;

    /// 卡牌稀有度
    const RARITY_COMMON: u8 = 0;
    const RARITY_UNCOMMON: u8 = 1;
    const RARITY_RARE: u8 = 2;
    const RARITY_LEGENDARY: u8 = 3;

    /// 错误代码
    const EInvalidRarity: u64 = 0;
    const ENotOwner: u64 = 1;
    const EInvalidLevel: u64 = 2;

    /// 卡牌对象
    public struct Card has key, store {
        id: UID,
        name: String,
        rarity: u8,
        image_url: String,
        level: u8,
        owner: address,
        created_at: u64
    }

    /// 卡牌创建事件
    public struct CardCreated has copy, drop {
        card_id: address,
        name: String,
        rarity: u8,
        owner: address,
        created_at: u64
    }

    /// 卡牌升级事件
    public struct CardUpgraded has copy, drop {
        card_id: address,
        old_level: u8,
        new_level: u8,
        upgraded_at: u64
    }

    /// 卡牌销毁事件
    public struct CardBurned has copy, drop {
        card_id: address,
        owner: address,
        burned_at: u64
    }

    /// 创建新卡牌
    public fun create_card(
        name: String,
        rarity: u8,
        image_url: String,
        ctx: &mut TxContext
    ): Card {
        assert!(rarity <= RARITY_LEGENDARY, EInvalidRarity);
        
        let sender = tx_context::sender(ctx);
        let now = tx_context::epoch_timestamp_ms(ctx);
        
        let card = Card {
            id: object::new(ctx),
            name,
            rarity,
            image_url,
            level: 0,
            owner: sender,
            created_at: now
        };

        event::emit(CardCreated {
            card_id: object::uid_to_address(&card.id),
            name,
            rarity,
            owner: sender,
            created_at: now
        });

        card
    }

    /// 升级卡牌
    public fun upgrade_card(card: &mut Card, ctx: &mut TxContext) {
        assert!(card.owner == tx_context::sender(ctx), ENotOwner);
        assert!(card.level < 3, EInvalidLevel);  // 最高3级

        let old_level = card.level;
        card.level = card.level + 1;

        event::emit(CardUpgraded {
            card_id: object::uid_to_address(&card.id),
            old_level,
            new_level: card.level,
            upgraded_at: tx_context::epoch_timestamp_ms(ctx)
        });
    }

    /// 销毁卡牌
    public fun burn_card(card: Card, ctx: &mut TxContext): (u8, address) {
        let Card { id, name: _, rarity, image_url: _, level: _, owner, created_at: _ } = card;
        
        let card_id = object::uid_to_address(&id);
        
        event::emit(CardBurned {
            card_id,
            owner,
            burned_at: tx_context::epoch_timestamp_ms(ctx)
        });

        object::delete(id);
        
        // 返回稀有度和所有者，便于碎片计算和转账
        (rarity, owner)
    }

    /// 转移卡牌所有权
    public entry fun transfer_card(card: Card, recipient: address) {
        transfer::public_transfer(card, recipient);
    }

    /// 获取卡牌ID
    public fun get_id(card: &Card): &UID {
        &card.id
    }

    // Getters
    public fun get_name(card: &Card): String { card.name }
    public fun get_rarity(card: &Card): u8 { card.rarity }
    public fun get_level(card: &Card): u8 { card.level }
    public fun get_owner(card: &Card): address { card.owner }
    public fun get_image_url(card: &Card): String { card.image_url }
} 