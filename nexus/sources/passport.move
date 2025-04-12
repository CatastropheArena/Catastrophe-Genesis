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

    //---------------------------------------------- Events ----------------------------------------------//
    /// 护照创建事件
    public struct PassportCreated has copy, drop {
        passport_id: address,
        owner: address,
        created_at: u64
    }

    /// 每日奖励领取事件
    public struct DailyRewardsClaimed has copy, drop {
        passport_id: address,
        owner: address,
        amount: u64,
        claimed_at: u64
    }

    //---------------------------------------------- User functions ----------------------------------------------//
    /// 创建新的护照
    public(package) fun create_passport(
        state: &mut State,
        clock: &Clock, 
        ctx: &mut TxContext
    ){
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
        event::emit(PassportCreated {
            passport_id: object::uid_to_address(&passport.id),
            owner: sender,
            created_at: now
        });
        transfer::transfer(passport, sender);

        vector::push_back(&mut state.users, sender);

    }

    /// 领取每日奖励
    public(package) fun claim_daily_rewards(
        passport: &mut Passport, 
        clock: &Clock, 
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);
        
        let now = clock::timestamp_ms(clock);
        assert!(can_claim_daily_rewards(passport, clock), ETooEarlyToClaim);

        passport.last_claim_time = now;
        passport.daily_rewards_claimed = passport.daily_rewards_claimed + 1;

        event::emit(DailyRewardsClaimed {
            passport_id: object::uid_to_address(&passport.id),
            owner: sender,
            amount: passport.daily_rewards_claimed,
            claimed_at: now
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

    //---------------------------------------------- Helper functions ----------------------------------------------//

}