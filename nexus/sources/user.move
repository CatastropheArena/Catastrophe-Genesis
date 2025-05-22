module nexus::user{
    use sui::clock::{Clock};
    use sui::coin::{Self, Coin};
    use std::string;
    use nexus::treasury::{Self, Treasury};
    use nexus::passport::{Self, State, Passport};
    use nexus::fragment::{Self, FragmentStore};
    use nexus::fish::{FISH};
    use sui::sui::SUI;

    //---------------------------------------------- Error Codes ----------------------------------------------//
    const EIncorrectPaymentAmount: u64 = 0;

    //---------------------------------------------- Consts ----------------------------------------------//
    const FRAGMENT_TO_COIN_RATE: u64 = 2; // 100 coin :: 50 Fragments
    const FISH_TO_SUI_RATE: u64 = 100; // 100 FISH :: 1 SUI

    //---------------------------------------------- Entry functions ----------------------------------------------//
    public entry fun create_new_user(
        state: &mut State,
        treasury: &mut Treasury,
        store: &mut FragmentStore,
        clock: &Clock, 
        ctx: &mut TxContext
    ){
        let sender = tx_context::sender(ctx);
        
        // 创建护照
        let passport = passport::create_passport(state, clock, ctx);
        
        // 发放初始奖励（碎片和FISH）
        let (fragments, fish) = treasury::distribute_initial_rewards_all(treasury, store, &passport, clock, ctx);
        
        // 处理碎片奖励
        let req = fragment::transfer_fragments(fragments, sender, ctx);
        fragment::confirm_request(store, req, ctx);
        
        // 转移 FISH 和护照给用户
        transfer::public_transfer(fish, sender);
        passport::transfer_passport(passport, ctx);
    }

    public entry fun claim_daily_rewards(
        passport: &mut Passport, 
        treasury: &mut Treasury,
        store: &mut FragmentStore,
        clock: &Clock, 
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);

        // 检查并更新护照状态
        passport::claim_daily_rewards(passport, clock);

        // 发放每日奖励
        let fragments = treasury::distribute_daily_rewards(treasury, store, passport, clock, ctx);
        let req = fragment::transfer_fragments(fragments, sender, ctx);
        fragment::confirm_request(store, req, ctx);
    }

    public entry fun buy_fragments(
        payment: Coin<FISH>,
        amount: u64,
        store: &mut FragmentStore,
        treasury: &mut Treasury,
        clock: &Clock,
        ctx: &mut TxContext
    ){
        assert!(coin::value(&payment) == amount * FRAGMENT_TO_COIN_RATE, EIncorrectPaymentAmount);
        treasury::deposit(
            treasury, 
            payment, 
            string::utf8(b"Buy fragments."), 
            clock, 
            ctx
        );
        fragment::buy_fragments(amount, store, clock, ctx);
    }

    public fun buy_fish(
      treasury: &mut Treasury,
      payment: Coin<SUI>,
      amount: u64,
      clock: &Clock,
      ctx: &mut TxContext,
    ){
      let sender = tx_context::sender(ctx);
      assert!(coin::value(&payment)/1_000_000_000 == amount/FISH_TO_SUI_RATE, EIncorrectPaymentAmount);
      treasury::deposit_sui(treasury, payment, clock, ctx);
      transfer::public_transfer(treasury::withdraw(treasury, amount, ctx), sender);
    }
}