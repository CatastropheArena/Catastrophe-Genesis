#[allow(unused_use)]
module nexus::fish {
    use sui::event;
    use sui::clock::{Self, Clock};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin, TreasuryCap};
    use std::string::{Self, String};

    //---------------------------------------------- Struct ----------------------------------------------//
    /// One-time witness for the module
    public struct FISH has drop {}

    //---------------------------------------------- Functions ----------------------------------------------//
    /// 初始化模块
    fun init(witness: FISH, ctx: &mut TxContext) {
      let (mut treasury, metadata) = coin::create_currency(
        witness,
        3,
        b"FISH",
        b"FISH",
        b"Fish coin to be used in Catastrophe Genesis",
        option::none(), //todo: add
        ctx,
      );
      mint(&mut treasury, 1_000_000, ctx.sender(), ctx);
      transfer::public_freeze_object(metadata);
      transfer::public_transfer(treasury, ctx.sender())

    }

    public fun mint(
      treasury_cap: &mut TreasuryCap<FISH>,
      amount: u64,
      recipient: address,
      ctx: &mut TxContext,
    ) {
      let coin = coin::mint(treasury_cap, amount, ctx);
      transfer::public_transfer(coin, recipient)
    }


} 