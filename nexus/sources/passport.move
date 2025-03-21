module nexus::passport {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::clock::{Self, Clock};
    use sui::table::{Self, Table};
    use sui::event;
    use std::string::{Self, String};
    use std::vector;

    /// 护照对象，每个地址只能拥有一个
    struct Passport has key, store {
        id: UID,
        owner: address,
        last_claim_time: u64,
        daily_rewards_claimed: u64,
        rental_cards: vector<address>,
        created_at: u64
    }

    /// 护照创建事件
    struct PassportCreated has copy, drop {
        passport_id: address,
        owner: address,
        created_at: u64
    }

    /// 每日奖励领取事件
    struct DailyRewardsClaimed has copy, drop {
        passport_id: address,
        owner: address,
        amount: u64,
        claimed_at: u64
    }

    // 错误代码
    const EPassportAlreadyExists: u64 = 0;
    const ENotOwner: u64 = 1;
    const ETooEarlyToClaim: u64 = 2;

    // 常量
    const DAY_IN_MS: u64 = 86400000; // 24 * 60 * 60 * 1000 毫秒

    /// 创建新的护照
    public entry fun create_passport(clock: &Clock, ctx: &mut TxContext) {
        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        
        let passport = Passport {
            id: object::new(ctx),
            owner: sender,
            last_claim_time: now,
            daily_rewards_claimed: 0,
            rental_cards: vector::empty(),
            created_at: now
        };

        event::emit(PassportCreated {
            passport_id: object::uid_to_address(&passport.id),
            owner: sender,
            created_at: now
        });

        transfer::public_transfer(passport, sender);
    }

    /// 检查是否可以领取每日奖励
    public fun can_claim_daily_rewards(passport: &Passport, clock: &Clock): bool {
        let now = clock::timestamp_ms(clock);
        now >= passport.last_claim_time + DAY_IN_MS
    }

    /// 领取每日奖励
    public fun claim_daily_rewards(passport: &mut Passport, clock: &Clock, ctx: &mut TxContext) {
        let sender = tx_context::sender(ctx);
        assert!(passport.owner == sender, ENotOwner);
        
        let now = clock::timestamp_ms(clock);
        assert!(now >= passport.last_claim_time + DAY_IN_MS, ETooEarlyToClaim);

        passport.last_claim_time = now;
        passport.daily_rewards_claimed = passport.daily_rewards_claimed + 1;

        event::emit(DailyRewardsClaimed {
            passport_id: object::uid_to_address(&passport.id),
            owner: sender,
            amount: 1,
            claimed_at: now
        });
    }

    /// 添加租赁卡牌
    public fun add_rental_card(passport: &mut Passport, card_address: address) {
        vector::push_back(&mut passport.rental_cards, card_address);
    }

    /// 移除租赁卡牌
    public fun remove_rental_card(passport: &mut Passport, card_address: address) {
        let (exists, index) = vector::index_of(&passport.rental_cards, &card_address);
        if (exists) {
            vector::remove(&mut passport.rental_cards, index);
        };
    }

    /// 获取护照拥有者
    public fun get_owner(passport: &Passport): address {
        passport.owner
    }

    /// 获取租赁卡牌列表
    public fun get_rental_cards(passport: &Passport): &vector<address> {
        &passport.rental_cards
    }
}