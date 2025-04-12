module nexus::admin {
    use sui::clock::{Clock};
    use nexus::treasury::{Self, Treasury};

    //---------------------------------------------- Entry functions ----------------------------------------------//
    public entry fun change_admin(
        treasury: &mut Treasury,
        new_admin: address,
        clock: &Clock,
        ctx: &TxContext
    ){
        treasury::change_admin(treasury, new_admin, clock, ctx);
    }

    public entry fun withdraw_coin(
        treasury: &mut Treasury,
        amount: u64,
        ctx: &mut TxContext
    ){
        transfer::public_transfer(
            treasury::withdraw(treasury, amount, ctx), 
            tx_context::sender(ctx)
        )
    }
}