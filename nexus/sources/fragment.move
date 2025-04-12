#[allow(unused_use)]
module nexus::fragment {
    use sui::event;
    use sui::coin::{Self, TreasuryCap};
    use sui::token::{Self, Token, TokenPolicyCap, ActionRequest};
    use sui::balance::{Self, Balance};
    use sui::sui::SUI;
    use std::string::{Self, String};
    use sui::clock::{Self, Clock};

    //---------------------------------------------- Structs ----------------------------------------------//
    /// One-time witness for the module
    public struct FRAGMENT has drop {}

    /// Fragment coin metadata
    public struct FragmentMetadata has key, store {
        id: UID
    }

    public struct FragmentStore has key {
        id: UID,
        profits: Balance<SUI>,
        treasury: TreasuryCap<FRAGMENT>,
    }

    //---------------------------------------------- Events ----------------------------------------------//

    /// 碎片铸造事件
    public struct FragmentMinted has copy, drop {
        amount: u64,
        at: u64
    }
    
    /// 碎片使用事件
    public struct FragmentSpent has copy, drop {
        amount: u64,
        owner: address,
        usage: String,
        at: u64
    }

    /// 碎片转移事件
    public struct FragmentTransferred has copy, drop {
        amount: u64,
        from: address,
        to: address,
    }

    //---------------------------------------------- Init ----------------------------------------------//
    fun init(witness: FRAGMENT, ctx: &mut TxContext) {
        // 创建碎片代币
        let (treasury_cap, metadata) = coin::create_currency(
            witness,
            0, // 精度
            b"FT",
            b"Fragment",
            b"Game card fragments for synthesis, upgrade and in-game usage",
            option::none(), //todo: add
            ctx
        );
        
        let (mut policy, cap) = token::new_policy(&treasury_cap, ctx);
        token::allow(&mut policy, &cap, token::spend_action(), ctx);
        token::allow(&mut policy, &cap, token::transfer_action(), ctx);

        transfer::share_object(FragmentStore{
            id: object::new(ctx),
            profits: balance::zero(),
            treasury: treasury_cap
        });
        
        // 发布元数据为不可变对象
        transfer::public_freeze_object(metadata);
        // 转移铸币权给部署者
        transfer::public_transfer(cap, tx_context::sender(ctx));
        token::share_policy(policy);
    }

    #[test_only]
    public fun init_for_testing(ctx: &mut TxContext) {
        init(FRAGMENT {}, ctx)
    }

    //---------------------------------------------- Core functions ----------------------------------------------//
    public(package) fun buy_fragments(
        amount: u64,
        store: &mut FragmentStore,
        clock: &Clock,
        ctx: &mut TxContext
    ){
        let sender = tx_context::sender(ctx);
        let fragments = mint(store, amount, clock, ctx);
        let req = transfer_fragments(fragments, sender, ctx);
        confirm_request(store, req, ctx);
    }

    /// 管理员铸造碎片
    public(package) fun mint(
        store: &mut FragmentStore,
        amount: u64, 
        clock: &Clock,
        ctx: &mut TxContext
    ): Token<FRAGMENT> {
        let fragments = token::mint(&mut store.treasury, amount, ctx);
        
        event::emit(FragmentMinted {
            amount,
            at: clock::timestamp_ms(clock)
        });
        fragments
    }

    public(package) fun spend_fragments(
        fragments: Token<FRAGMENT>, 
        usage: String,
        clock: &Clock,
        ctx: &mut TxContext
    ): ActionRequest<FRAGMENT> {
        event::emit(FragmentSpent {
            amount: token::value(&fragments),
            owner: tx_context::sender(ctx),
            usage,
            at: clock::timestamp_ms(clock)
        });
        token::spend(fragments, ctx)
    }
 

    /// 转移碎片
    public fun transfer_fragments(
        fragments: Token<FRAGMENT>, 
        recipient: address, 
        ctx: &mut TxContext
    ): ActionRequest<FRAGMENT> {
        let amount = token::value(&fragments);
        let sender = tx_context::sender(ctx);

        event::emit(FragmentTransferred {
            amount,
            from: sender,
            to: recipient,
        });

        token::transfer(fragments, recipient, ctx)
    }

    public(package) fun confirm_request( //todo: does this require package visibility?
        store: &mut FragmentStore,
        request: ActionRequest<FRAGMENT>,
        ctx: &mut TxContext
    ){
        token::confirm_with_treasury_cap(&mut store.treasury, request, ctx);
    }
    //---------------------------------------------- Getter functions ----------------------------------------------//
    /// 获取碎片余额
    public fun balance(fragments: &Token<FRAGMENT>): u64 {
        token::value(fragments)
    }
} 