module nexus::game{
    use sui::clock::{Self, Clock};
    use sui::coin::{Self, Coin};
    use std::string::{String};
    use nexus::passport::{Self, Passport};
    use nexus::treasury::{Self, Treasury};
    use nexus::fish::FISH;

    //---------------------------------------------- Error Codes ----------------------------------------------//
    const EIncorrectPaymentAmount: u64 = 0;
    
    //---------------------------------------------- Consts ----------------------------------------------//
    const GAME_COST: u64 = 100;

    //---------------------------------------------- Structs ----------------------------------------------//
    public struct GameEntry has key {
        id: UID,
        user_passport: address,
        game_id: address,
        timestamp: u64,
    }

    //---------------------------------------------- Events ----------------------------------------------//
    public struct PassportCreated has copy, drop {
        passport_id: address,
        owner: address,
        created_at: u64,
        initial_claim_time: u64,
        total_users: u64  // 当前系统中的总用户数
    }

    //---------------------------------------------- Entry functions ----------------------------------------------//
    public entry fun join_game(
        passport: &Passport,
        treasury: &mut Treasury,
        payment: Coin<FISH>,
        game: address, 
        purpose: String,
        clock: &Clock,
        ctx: &mut TxContext,
    ){
        let sender = tx_context::sender(ctx);
        let user_passport = passport::get_passport_id(passport);
        assert!(coin::value(&payment) == GAME_COST, EIncorrectPaymentAmount);
        treasury::deposit(
            treasury,
            payment,
            purpose,
            clock,
            ctx
        );
        transfer::transfer(GameEntry{
            id: object::new(ctx),
            user_passport,
            game_id: game,
            timestamp: clock::timestamp_ms(clock),
        }, sender);
    }
}