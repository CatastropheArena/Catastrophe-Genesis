module nexus::user{
    use sui::clock::{Clock};
    use sui::coin::{Self, Coin};
    use std::string;
    use nexus::treasury::{Self, Treasury};
    use nexus::passport::{Self, State, Passport};
    use nexus::fragment::{Self, FragmentStore};
    use nexus::fish::{FISH};

    //---------------------------------------------- Error Codes ----------------------------------------------//
    const EIncorrectPaymentAmount: u64 = 0;

    //---------------------------------------------- Consts ----------------------------------------------//
    const FRAGMENT_TO_COIN_RATE: u64 = 2; // 100 coin :: 50 Fragments

    //---------------------------------------------- Entry functions ----------------------------------------------//
    public entry fun create_new_user(
        state: &mut State,
        treasury: &mut Treasury,
        store: &mut FragmentStore,
        clock: &Clock, 
        ctx: &mut TxContext
    ){
        let sender = tx_context::sender(ctx);
        passport::create_passport(state, clock, ctx);
        let fragments = treasury::distribute_initial_rewards(treasury, store, sender, clock, ctx);
        let req = fragment::transfer_fragments(fragments, sender, ctx);
        fragment::confirm_request(store, req, ctx);
    }

    public entry fun claim_daily_rewards(
        passport: &mut Passport, 
        treasury: &mut Treasury,
        store: &mut FragmentStore,
        clock: &Clock, 
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);

        passport::claim_daily_rewards(passport, clock, ctx);

        let fragments = treasury::distribute_daily_rewards(treasury, store, sender, clock, ctx);
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

    
}