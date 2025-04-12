#[allow(unused_use, unused_mut_parameter, unused_variable, )]
module nexus::treasury {
    //---------------------------------------------- Dependencies ----------------------------------------------//
    use sui::event;
    use sui::clock::{Self, Clock};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin, TreasuryCap};
    use sui::token::{Self, Token};
    use std::string::{Self, String};
    use nexus::fragment::{Self, FRAGMENT, FragmentStore};
    use nexus::fish::{Self, FISH};

    //---------------------------------------------- Error Codes ----------------------------------------------//
    const EInsufficientBalance: u64 = 0;
    const ENotAuthorized: u64 = 1;
    const EInvalidAddress: u64 = 2;

    //---------------------------------------------- Struct ----------------------------------------------//
    // /// One-time witness for the module
    // public struct TREASURY has drop {}

    /// 资金库对象
    public struct Treasury has key {
        id: UID,
        admin: address,
        coin_balance: Balance<FISH>,
    }

    //---------------------------------------------- Events ----------------------------------------------//
    /// 奖励发放事件
    public struct RewardsDistributed has copy, drop {
        treasury_id: address,
        recipient: address,
        amount: u64,
        distributed_at: u64
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

    /// 发放初始奖励
    public(package) fun distribute_initial_rewards(
        treasury: &mut Treasury,
        store: &mut FragmentStore,
        recipient: address,
        clock: &Clock,
        ctx: &mut TxContext
    ): Token<FRAGMENT> {
        // 初始奖励: 50个碎片
        let amount = 50;
        let rewards = fragment::mint(store, amount, clock, ctx);

        event::emit(RewardsDistributed {
            treasury_id: object::uid_to_address(&treasury.id),
            recipient,
            amount,
            distributed_at: clock::timestamp_ms(clock)
        });

        rewards
    }

    /// 发放每日奖励
    public(package) fun distribute_daily_rewards(
        treasury: &mut Treasury,
        store: &mut FragmentStore,
        recipient: address,
        clock: &Clock,
        ctx: &mut TxContext
    ): Token<FRAGMENT> {

        // 每日奖励: 10个碎片
        let amount = 10;
        let rewards = fragment::mint(store, amount, clock, ctx);

        event::emit(RewardsDistributed {
            treasury_id: object::uid_to_address(&treasury.id),
            recipient,
            amount,
            distributed_at: clock::timestamp_ms(clock)
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