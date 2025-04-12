#[allow(unused_mut_parameter)]
module nexus::passport {
    use sui::clock::{Self, Clock};
    use sui::event;

    //---------------------------------------------- Consts ----------------------------------------------//
    const DAY_IN_MS: u64 = 86400000; // 24 * 60 * 60 * 1000 毫秒

    //---------------------------------------------- Errors ----------------------------------------------//
    const EPassportAlreadyExists: u64 = 0;
    const ETooEarlyToClaim: u64 = 1;

    //---------------------------------------------- Structs ----------------------------------------------//
    public struct State has key {
        id: UID,
        users: vector<address>
    }

    /// 护照对象，每个地址只能拥有一个
    public struct Passport has key {
        id: UID,
        owner: address,
        last_claim_time: u64,
        daily_rewards_claimed: u64,
        rental_cards: vector<address>,
        created_at: u64
    }

    //---------------------------------------------- Init ----------------------------------------------//
    fun init(ctx: &mut TxContext){
        transfer::share_object(State{
            id: object::new(ctx),
            users: vector::empty()
        })
    }

    #[test_only]
    public fun init_for_testing(ctx: &mut TxContext) {
        init(ctx)
    }

    //---------------------------------------------- Events ----------------------------------------------//
    /// 护照创建事件
    public struct PassportCreated has copy, drop {
        passport_id: address,
        owner: address,
        created_at: u64,
        initial_claim_time: u64,
        total_users: u64  // 当前系统中的总用户数
    }

    /// 每日奖励领取事件
    public struct DailyRewardsClaimed has copy, drop {
        passport_id: address,
        owner: address,
        claim_count: u64,      // 当前是第几次领取
        last_claim_time: u64,  // 上次领取时间
        current_claim_time: u64, // 本次领取时间
        next_claim_time: u64    // 下次可领取时间
    }

    /// 护照状态更新事件
    public struct PassportStatusUpdated has copy, drop {
        passport_id: address,
        owner: address,
        daily_rewards_claimed: u64,
        rental_cards_count: u64,
        updated_at: u64
    }

    //---------------------------------------------- User functions ----------------------------------------------//
    /// 创建新的护照
    public(package) fun create_passport(
        state: &mut State,
        clock: &Clock, 
        ctx: &mut TxContext
    ): Passport {
        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        assert!(!vector::contains(&state.users, &sender), EPassportAlreadyExists);
        
        let passport = Passport {
            id: object::new(ctx),
            owner: sender,
            last_claim_time: now,
            daily_rewards_claimed: 0,
            rental_cards: vector::empty(),
            created_at: now
        };

        let passport_id = object::uid_to_address(&passport.id);
        vector::push_back(&mut state.users, sender);
        
        event::emit(PassportCreated {
            passport_id,
            owner: sender,
            created_at: now,
            initial_claim_time: now,
            total_users: vector::length(&state.users)
        });

        event::emit(PassportStatusUpdated {
            passport_id,
            owner: sender,
            daily_rewards_claimed: 0,
            rental_cards_count: 0,
            updated_at: now
        });

        passport
    }

    /// 转移护照
    public(package) fun transfer_passport(
        passport: Passport,
        ctx: &mut TxContext
    ){
        transfer::transfer(passport, ctx.sender());
    }

    /// 领取每日奖励
    public(package) fun claim_daily_rewards(
        passport: &mut Passport, 
        clock: &Clock
    ) {
        let now = clock::timestamp_ms(clock);
        assert!(can_claim_daily_rewards(passport, clock), ETooEarlyToClaim);

        let passport_id = object::uid_to_address(&passport.id);
        let last_claim = passport.last_claim_time;
        
        passport.last_claim_time = now;
        passport.daily_rewards_claimed = passport.daily_rewards_claimed + 1;

        event::emit(DailyRewardsClaimed {
            passport_id,
            owner: passport.owner,
            claim_count: passport.daily_rewards_claimed,
            last_claim_time: last_claim,
            current_claim_time: now,
            next_claim_time: now + DAY_IN_MS
        });

        event::emit(PassportStatusUpdated {
            passport_id,
            owner: passport.owner,
            daily_rewards_claimed: passport.daily_rewards_claimed,
            rental_cards_count: vector::length(&passport.rental_cards),
            updated_at: now
        });
    }

    // // todo: rent instead of add
    // /// 添加租赁卡牌
    // public fun add_rental_card(passport: &mut Passport, card_address: address) {
    //     vector::push_back(&mut passport.rental_cards, card_address);
    // }

    // // todo: remove expired rental card
    // /// 移除租赁卡牌
    // public fun remove_rental_card(passport: &mut Passport, card_address: address) {
    //     let (exists, index) = vector::index_of(&passport.rental_cards, &card_address);
    //     if (exists) {
    //         vector::remove(&mut passport.rental_cards, index);
    //     };
    // }

    //---------------------------------------------- Getter functions ----------------------------------------------//
    /// 检查是否可以领取每日奖励
    public fun can_claim_daily_rewards(passport: &Passport, clock: &Clock): bool {
        let now = clock::timestamp_ms(clock);
        now >= passport.last_claim_time + DAY_IN_MS
    }

    /// 获取护照拥有者
    public fun get_owner(passport: &Passport): address {
        passport.owner
    }

    /// 获取租赁卡牌列表
    public fun get_rental_cards(passport: &Passport): &vector<address> {
        &passport.rental_cards
    }

    /// 获取护照 ID 地址
    public fun get_passport_id(passport: &Passport): address {
        object::uid_to_address(&passport.id)
    }

    /// 获取每日奖励领取次数,记录用户的打卡次数
    public fun get_daily_rewards_claimed(passport: &Passport): u64 {
        passport.daily_rewards_claimed
    }

    //---------------------------------------------- Helper functions ----------------------------------------------//

}