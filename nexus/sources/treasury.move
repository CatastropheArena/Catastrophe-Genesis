#[allow(unused_use, unused_mut_parameter, unused_variable, )]
module nexus::treasury {
    //---------------------------------------------- Dependencies ----------------------------------------------//
    use sui::event;
    use sui::clock::{Self, Clock};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin, TreasuryCap};
    use sui::token::{Self, Token};
    use std::string::{Self, String};
    use sui::vec_set::{Self, VecSet};
    use nexus::fragment::{Self, FRAGMENT, FragmentStore};
    use nexus::fish::{Self, FISH};
    use nexus::passport::{Self, Passport};

    //---------------------------------------------- Error Codes ----------------------------------------------//
    const EInsufficientBalance: u64 = 0;
    const ENotAuthorized: u64 = 1;
    const EInvalidAddress: u64 = 2;
    const EAlreadyClaimed: u64 = 3;

    //---------------------------------------------- Struct ----------------------------------------------//
    /// 资金库对象
    public struct Treasury has key {
        id: UID,
        admin: address,
        coin_balance: Balance<FISH>,
        claimed_passports: VecSet<ID>, // 记录已领取初始奖励的护照ID
    }

    //---------------------------------------------- Events ----------------------------------------------//
    /// 奖励发放事件
    public struct RewardsDistributed has copy, drop {
        treasury_id: address,
        passport_id: address,
        recipient: address,
        amount: u64,
        reward_type: String,  // 奖励类型: "Initial Fragment", "Initial FISH", "Daily Fragment" 等
        distributed_at: u64,
        total_claimed_count: u64,  // 该护照已领取奖励的次数
        balance_after: u64,   // Treasury 剩余 FISH 数量
        purpose: String      // 发放目的说明
    }

    /// 资金存入事件
    public struct FundsDeposited has copy, drop {
        treasury_id: address,
        depositor: address,
        amount: u64,
        purpose: String,
        deposited_at: u64
    }

    public struct AdminChanged has copy, drop {
        old_admin: address,
        new_admin: address,
        at: u64
    }

    //---------------------------------------------- Init ----------------------------------------------//
    fun init(ctx: &mut TxContext) {
        let sender = tx_context::sender(ctx);
        let treasury = Treasury {
            id: object::new(ctx),
            admin: sender,
            coin_balance: balance::zero<FISH>(),
            claimed_passports: vec_set::empty(),
        };

        transfer::share_object(treasury);
    }

    //---------------------------------------------- User functions ----------------------------------------------//
    /// 存入资金
    public fun deposit(
        treasury: &mut Treasury,
        payment: Coin<FISH>,
        purpose: String,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let amount = coin::value(&payment);
        let depositor = tx_context::sender(ctx);

        balance::join(&mut treasury.coin_balance, coin::into_balance(payment));

        event::emit(FundsDeposited {
            treasury_id: object::uid_to_address(&treasury.id),
            depositor,
            amount,
            purpose,
            deposited_at: clock::timestamp_ms(clock)
        });
    }

    /// 发放初始奖励（包含碎片和FISH）
    public(package) fun distribute_initial_rewards_all(
        treasury: &mut Treasury,
        store: &mut FragmentStore,
        passport: &Passport,
        clock: &Clock,
        ctx: &mut TxContext
    ): (Token<FRAGMENT>, Coin<FISH>) {
        let passport_id = object::id(passport);
        assert!(!vec_set::contains(&treasury.claimed_passports, &passport_id), EAlreadyClaimed);
        
        // 记录护照已领取
        vec_set::insert(&mut treasury.claimed_passports, passport_id);
        
        let recipient = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        let passport_id_addr = passport::get_passport_id(passport);
        
        // 发放初始碎片: 50个碎片
        let fragment_amount = 50;
        let rewards = fragment::mint(store, fragment_amount, clock, ctx);

        event::emit(RewardsDistributed {
            treasury_id: object::uid_to_address(&treasury.id),
            passport_id: passport_id_addr,
            recipient,
            amount: fragment_amount,
            reward_type: string::utf8(b"Initial Fragment"),
            distributed_at: now,
            total_claimed_count: vec_set::size(&treasury.claimed_passports),
            balance_after: balance::value(&treasury.coin_balance),
            purpose: string::utf8(b"Initial rewards distribution - Fragment part")
        });

        // 发放初始 FISH: 100 FISH
        let fish_amount = 100;
        let fish = coin::from_balance(balance::split(&mut treasury.coin_balance, fish_amount), ctx);

        event::emit(RewardsDistributed {
            treasury_id: object::uid_to_address(&treasury.id),
            passport_id: passport_id_addr,
            recipient,
            amount: fish_amount,
            reward_type: string::utf8(b"Initial FISH"),
            distributed_at: now,
            total_claimed_count: vec_set::size(&treasury.claimed_passports),
            balance_after: balance::value(&treasury.coin_balance),
            purpose: string::utf8(b"Initial rewards distribution - FISH part")
        });

        (rewards, fish)
    }

    /// 发放每日奖励
    public(package) fun distribute_daily_rewards(
        treasury: &mut Treasury,
        store: &mut FragmentStore,
        passport: &Passport,
        clock: &Clock,
        ctx: &mut TxContext
    ): Token<FRAGMENT> {
        let recipient = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        let passport_id_addr = passport::get_passport_id(passport);
        
        // 每日奖励: 10个碎片
        let amount = 10;
        let rewards = fragment::mint(store, amount, clock, ctx);

        event::emit(RewardsDistributed {
            treasury_id: object::uid_to_address(&treasury.id),
            passport_id: passport_id_addr,
            recipient,
            amount,
            reward_type: string::utf8(b"Daily Fragment"),
            distributed_at: now,
            total_claimed_count: passport::get_daily_rewards_claimed(passport),
            balance_after: balance::value(&treasury.coin_balance),
            purpose: string::utf8(b"Daily rewards distribution")
        });
        
        rewards
    }

    //---------------------------------------------- Admin functions ----------------------------------------------//
    public(package) fun withdraw(
        treasury: &mut Treasury,
        amount: u64,
        ctx: &mut TxContext
    ): Coin<FISH> {
        let sender = tx_context::sender(ctx);
        assert!(balance::value(&treasury.coin_balance) >= amount, EInsufficientBalance);
        assert_admin(sender, treasury.admin);
        coin::from_balance(balance::split(&mut treasury.coin_balance, amount), ctx)
    }

    public(package) fun change_admin(
        treasury: &mut Treasury,
        new_admin: address,
        clock: &Clock,
        ctx: &TxContext
    ){
        let sender = tx_context::sender(ctx);
        let old_admin = treasury.admin;
        assert_admin(sender, old_admin);
        assert!(new_admin != @0x0, EInvalidAddress);
        assert!(new_admin != old_admin, EInvalidAddress);
        treasury.admin = new_admin;
        event::emit(AdminChanged{
            old_admin,
            new_admin,
            at: clock::timestamp_ms(clock)
        })
    }
    //---------------------------------------------- Get functions ----------------------------------------------//

    public fun get_balance(treasury: &Treasury): u64 {
        balance::value(&treasury.coin_balance)
    }

    public fun get_admin(treasury: &Treasury): address {
        treasury.admin
    }

    fun assert_admin(sender: address, admin: address){
        assert!(sender == admin, ENotAuthorized);
    }
} 