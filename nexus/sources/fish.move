#[allow(unused_use)]
module nexus::fish {
    use sui::event;
    use sui::clock::{Self, Clock};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin, TreasuryCap};
    use std::string::{Self, String};

    /// max total supply
    const SUPPLY: u64 = 1_000_000_000;

    /// error: exceed max supply
    const EExceedMaxSupply: u64 = 0;

    /// One-time witness for the module
    public struct FISH has drop {}

    /// 初始化模块
    fun init(witness: FISH, ctx: &mut TxContext) {
      let (treasury, metadata) = coin::create_currency(
        witness,
        6,
        b"FISH",
        b"FISH",
        b"Fish coin to be used in Catastrophe Genesis",
        option::none(), //todo: add
        ctx,
      );
      transfer::public_freeze_object(metadata);
      transfer::public_transfer(treasury, ctx.sender())
    }

    public fun mint(
      treasury_cap: &mut TreasuryCap<FISH>,
      amount: u64,
      recipient: address,
      ctx: &mut TxContext,
    ) {
      assert!(coin::total_supply(treasury_cap) + amount <= SUPPLY, EExceedMaxSupply);
      let coin = coin::mint(treasury_cap, amount, ctx);
      transfer::public_transfer(coin, recipient)
    }
} 