module nexus::game{
    use sui::clock::{Self, Clock};
    use nexus::passport::{Self, Passport};

    public struct GameEntry has key {
        id: UID,
        user_passport: address,
        game_id: address,
        timestamp: u64,
    }

    public fun join_game(
        passport: &Passport,
        game: address, 
        clock: &Clock,
        ctx: &mut TxContext,
    ): GameEntry{
        let user_passport = passport::get_passport_id(passport);
        GameEntry{
            id: object::new(ctx),
            user_passport,
            game_id: game,
            timestamp: clock::timestamp_ms(clock),
        }
    }
}