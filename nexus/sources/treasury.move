module nexus::treasury {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;
    use sui::clock::{Self, Clock};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin, TreasuryCap};
    use std::string::{Self, String};
    use nexus::fragment::{Self, FRAGMENT};
    use nexus::passport::{Self, Passport};

    /// 错误代码
    const EInsufficientBalance: u64 = 0;
    const ENotAuthorized: u64 = 1;

    /// One-time witness for the module
    struct TREASURY has drop {}

    /// 资金库对象
    struct Treasury has key {
        id: UID,
        admin: address,
        fragment_balance: Balance<FRAGMENT>,
        created_at: u64
    }

    /// 资金库创建事件
    struct TreasuryCreated has copy, drop {
        treasury_id: address,
        admin: address,
        created_at: u64
    }

    /// 奖励发放事件
    struct RewardsDistributed has copy, drop {
        treasury_id: address,
        recipient: address,
        amount: u64,
        distributed_at: u64
    }

    /// 资金存入事件
    struct FundsDeposited has copy, drop {
        treasury_id: address,
        depositor: address,
        amount: u64,
        deposited_at: u64
    }

    /// 初始化模块
    fun init(witness: TREASURY, ctx: &mut TxContext) {
        let sender = tx_context::sender(ctx);
        let treasury = Treasury {
            id: object::new(ctx),
            admin: sender,
            fragment_balance: balance::zero<FRAGMENT>(),
            created_at: tx_context::epoch_timestamp_ms(ctx)
        };

        event::emit(TreasuryCreated {
            treasury_id: object::uid_to_address(&treasury.id),
            admin: sender,
            created_at: treasury.created_at
        });

        transfer::share_object(treasury);
    }

    /// 存入资金
    public fun deposit(
        treasury: &mut Treasury,
        payment: Coin<FRAGMENT>,
        ctx: &mut TxContext
    ) {
        let amount = coin::value(&payment);
        let depositor = tx_context::sender(ctx);

        balance::join(&mut treasury.fragment_balance, coin::into_balance(payment));

        event::emit(FundsDeposited {
            treasury_id: object::uid_to_address(&treasury.id),
            depositor,
            amount,
            deposited_at: tx_context::epoch_timestamp_ms(ctx)
        });
    }

    /// 发放初始奖励
    public fun distribute_initial_rewards(
        treasury: &mut Treasury,
        passport: &Passport,
        ctx: &mut TxContext
    ): Coin<FRAGMENT> {
        let sender = tx_context::sender(ctx);
        assert!(passport::get_owner(passport) == sender, ENotAuthorized);

        // 初始奖励: 250个碎片(足够合成10个普通卡牌)
        let amount = 250;
        assert!(balance::value(&treasury.fragment_balance) >= amount, EInsufficientBalance);

        let rewards = coin::from_balance(balance::split(&mut treasury.fragment_balance, amount), ctx);

        event::emit(RewardsDistributed {
            treasury_id: object::uid_to_address(&treasury.id),
            recipient: sender,
            amount,
            distributed_at: tx_context::epoch_timestamp_ms(ctx)
        });

        rewards
    }

    /// 发放每日奖励
    public fun distribute_daily_rewards(
        treasury: &mut Treasury,
        passport: &Passport,
        ctx: &mut TxContext
    ): Coin<FRAGMENT> {
        let sender = tx_context::sender(ctx);
        assert!(passport::get_owner(passport) == sender, ENotAuthorized);

        // 每日奖励: 50个碎片
        let amount = 50;
        assert!(balance::value(&treasury.fragment_balance) >= amount, EInsufficientBalance);

        let rewards = coin::from_balance(balance::split(&mut treasury.fragment_balance, amount), ctx);

        event::emit(RewardsDistributed {
            treasury_id: object::uid_to_address(&treasury.id),
            recipient: sender,
            amount,
            distributed_at: tx_context::epoch_timestamp_ms(ctx)
        });

        rewards
    }

    // Admin functions
    public fun withdraw(
        treasury: &mut Treasury,
        amount: u64,
        ctx: &mut TxContext
    ): Coin<FRAGMENT> {
        let sender = tx_context::sender(ctx);
        assert!(sender == treasury.admin, ENotAuthorized);
        assert!(balance::value(&treasury.fragment_balance) >= amount, EInsufficientBalance);

        coin::from_balance(balance::split(&mut treasury.fragment_balance, amount), ctx)
    }

    // Getters
    public fun get_balance(treasury: &Treasury): u64 {
        balance::value(&treasury.fragment_balance)
    }

    public fun get_admin(treasury: &Treasury): address {
        treasury.admin
    }
} 