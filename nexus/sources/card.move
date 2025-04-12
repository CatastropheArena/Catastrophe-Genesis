#[allow(implicit_const_copy, unused_const)]
module nexus::card {
    //---------------------------------------------- Dependencies ----------------------------------------------//
    use sui::event;
    use std::string::{Self, String};
    use sui::random::{Random, new_generator, generate_u64_in_range};
    use sui::clock::{Self, Clock};
    use sui::coin::{Self, Coin};
    use sui::token::{Self, Token};
    use nexus::fish::{FISH};
    use nexus::fragment::{Self, FRAGMENT, FragmentStore};
    use nexus::treasury::{Self, Treasury};


    //---------------------------------------------- Error Codes ----------------------------------------------//
    const EInvalidLevel: u64 = 0;
    const EIncorrectPaymentAmount: u64 = 1;

    //---------------------------------------------- Consts ----------------------------------------------//
    // coin cost to draw gacha once
    const DRAW_COST: u64 = 1000;
    
    // upgrade fragment cost
    const UPGRADE_COST: vector<u64> = vector[7, 15, 30];

    // combine fragment cost
    const COMBINE_COST: u64 = 20;

    /// 卡牌稀有度
    const RARITY_COMMON: u8 = 70;
    const RARITY_UNCOMMON: u8 = 20;
    const RARITY_RARE: u8 = 9;
    const RARITY_LEGENDARY: u8 = 1;
    const DENOMINATOR: u8 = 100;

    // card type
    const CARD_NAME: vector<vector<u8>> = vector[
        b"ANTI_EXPLOSION_CARD",
        b"SHIELD_CARD",
        b"SINGLE_ATTACK_CARD",
        b"GROUP_ATTACK_CARD",
        b"SEE_THE_FUTURE_CARD",
        b"SHUFFLE_CARD",
        b"DOUBLE_REWARD_CARD",
        b"DISCOUNT_CARD",
    ];

    // todo: add walrus/ipfs link
    const CARD_IMAGE: vector<vector<u8>> = vector[
        b"ANTI_EXPLOSION_CARD",
        b"SHIELD_CARD",
        b"SINGLE_ATTACK_CARD",
        b"GROUP_ATTACK_CARD",
        b"SEE_THE_FUTURE_CARD",
        b"SHUFFLE_CARD",
        b"DOUBLE_REWARD_CARD",
        b"DISCOUNT_CARD",
    ];

    const CARD_DESC: vector<vector<u8>> = vector[
        b"Negates the next exploding kitten drawn.",
        b"Grants immunity from negative effects this turn.",
        b"Forces one player to take an extra turn.",
        b"Forces all other players to take an extra turn.",
        b"View the top 5 cards of the deck.",
        b"Shuffle the deck and set bomb position.",
        b"Doubles rewards received for surviving the round.",
        b"Reduces cost for activating cards.",
    ];

    const STRATEGY: vector<vector<u8>> = vector[
        b"Essential in high-risk situations.",
        b"Effective against aggressive opponents.",
        b"Target opponents holding many cards.",
        b"Disrupt multiple opponents simultaneously.",
        b"Reduces risk when drawing unknown cards.",
        b"Use to avoid dangerous upcoming cards.",
        b"Maximize long-term economic gains.",
        b"Useful for frequent card activations.",
    ];

    const USAGE_COST: vector<u64> = vector[10, 8, 5, 15, 3, 4, 10, 8];
    const DYNAMIC_COST: vector<u64> = vector[20, 15, 10, 25, 10, 15, 20, 15];

    //---------------------------------------------- Structs ----------------------------------------------//
    /// 卡牌对象
    public struct Card has key, store {
        id: UID,
        card_type: CardType,
        level: u8,
        owner: address,
        created_at: u64
    }

    public struct CardType has store, copy, drop{
        name: String,
        description: String, 
        strategy: String,
        image_url: String,
        rarity: u8,
        cost: u64,
        dynamic_cost: u64, //next use increase required fragment by n%
    }

    //---------------------------------------------- Events ----------------------------------------------//
    /// 卡牌创建事件
    public struct CardCreated has copy, drop {
        card_id: address,
        card_type: String,
        rarity: u8,
        owner: address,
        created_at: u64
    }

    /// 卡牌升级事件
    public struct CardUpgraded has copy, drop {
        card_id: address,
        old_level: u8,
        new_level: u8,
        upgraded_at: u64
    }

    /// 卡牌合并事件
    public struct CardSynthesized has copy, drop {
        old_cards: vector<address>,
        new_card: address,
        card_type: String,
        rarity: u8,
        level: u8,
        owner: address,
        created_at: u64
    }

    /// 卡牌销毁事件
    public struct CardBurned has copy, drop {
        card_id: address,
        owner: address,
        burned_at: u64
    }

    //---------------------------------------------- User functions ----------------------------------------------//
    /// 创建新卡牌 (gacha)
    entry fun draw_card(
        r: &Random, 
        payment: Coin<FISH>,
        treasury: &mut Treasury,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        assert!(coin::value(&payment) == DRAW_COST, EIncorrectPaymentAmount);

        let rand = generate_u64_in_range(&mut new_generator(r, ctx), 0, 100);
        let rarity;
        let card_id;
        if(rand <70){
            rarity = RARITY_COMMON;
            let dice = generate_u64_in_range(&mut new_generator(r, ctx), 0, 1);
            if(dice == 0){
                card_id = 5;
            }else{
                card_id = 4;
            }
        }else if(rand >=70 && rand <90){
            rarity = RARITY_UNCOMMON;
            let dice = generate_u64_in_range(&mut new_generator(r, ctx), 0, 1);
            if(dice == 0){
                card_id = 1;
            }else{
                card_id = 2;
            }
        }else if(rand >=90 && rand <99){
            rarity = RARITY_RARE;
            let dice = generate_u64_in_range(&mut new_generator(r, ctx), 0, 2);
            if(dice == 0){
                card_id = 6;
            }else if(dice == 1){
                card_id = 3;
            }else{
                card_id = 0;
            }
        }else{
            rarity = RARITY_LEGENDARY;
            card_id = 7;
        };

        let card_name = string::utf8(*vector::borrow(&CARD_NAME, card_id));
        let card_type = CardType{
            name: card_name,
            description: string::utf8(*vector::borrow(&CARD_DESC, card_id)),
            strategy: string::utf8(*vector::borrow(&STRATEGY, card_id)),
            image_url: string::utf8(*vector::borrow(&CARD_IMAGE, card_id)),
            rarity,
            cost: *vector::borrow(&USAGE_COST, card_id),
            dynamic_cost: *vector::borrow(&DYNAMIC_COST, card_id),
        };
        
        let card = Card {
            id: object::new(ctx),
            card_type,
            level: 0,
            owner: sender,
            created_at: now
        };

        treasury::deposit(
            treasury, 
            payment, 
            string::utf8(b"One gacha pull."), 
            clock, 
            ctx
        );

        event::emit(CardCreated {
            card_id: object::uid_to_address(&card.id),
            card_type: card_name,
            rarity,
            owner: sender,
            created_at: now
        });

        transfer::public_transfer(card, sender);
    }

    /// 升级卡牌
    entry fun upgrade_card(
        card: &mut Card, 
        fragments: Token<FRAGMENT>,
        store: &mut FragmentStore,
        r: &Random, 
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        assert!(card.level < 3, EInvalidLevel);  // 最高3级

        let old_level = card.level;
        assert!(token::value(&fragments) == *vector::borrow(&UPGRADE_COST, (old_level as u64)), EIncorrectPaymentAmount);

        let rand = generate_u64_in_range(&mut new_generator(r, ctx), 0, 100);
        // success-rate:
        // level 0->1: 80%
        // level 1->2: 50%
        // level 2->3: 20%
        if(old_level == 0){
            if(rand <= 80){
                card.level = card.level + 1;
            }
        }else if(old_level == 1){
            if(rand > 50){
                card.level = card.level + 1;
            }
        }else{ //old_level = 2
            if(rand > 80){
                card.level = card.level + 1;
            }
        };

        event::emit(CardUpgraded {
            card_id: object::uid_to_address(&card.id),
            old_level,
            new_level: card.level,
            upgraded_at: clock::timestamp_ms(clock)
        });

        // spend fragment
        let req = fragment::spend_fragments(
            fragments, 
            string::utf8(b"Upgrade card."),
            clock,
            ctx
        );
        fragment::confirm_request(store, req, ctx);
    }

    /// 升级卡牌
    entry fun combine_card(
        card1: Card,
        card2: Card, 
        card3: Card,  
        fragments: Token<FRAGMENT>,
        store: &mut FragmentStore,
        r: &Random, 
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        assert!(token::value(&fragments) == COMBINE_COST, EIncorrectPaymentAmount);

        // level: 0-9
        let total_card_level = card1.level + card2.level + card3.level;
        // rarity: 3-15
        let total_card_rarity = rarity_mapping(card1.card_type.rarity) + 
            rarity_mapping(card2.card_type.rarity) + rarity_mapping(card3.card_type.rarity);

        let mut rand = generate_u64_in_range(&mut new_generator(r, ctx), 0, 100);
        let synthesis_card_level = synthesis_level_output(total_card_level, rand);
        rand = generate_u64_in_range(&mut new_generator(r, ctx), 0, 100);
        let synthesis_card_rarity = synthesis_rarity_output(total_card_rarity, rand);
        let i = get_vector_id(synthesis_card_rarity, r, ctx);

        let card_name = string::utf8(*vector::borrow(&CARD_NAME, i));
        let card_type = CardType{
            name: card_name,
            description: string::utf8(*vector::borrow(&CARD_DESC, i)),
            strategy: string::utf8(*vector::borrow(&STRATEGY, i)),
            image_url: string::utf8(*vector::borrow(&CARD_IMAGE, i)),
            rarity: synthesis_card_rarity,
            cost: *vector::borrow(&USAGE_COST, i),
            dynamic_cost: *vector::borrow(&DYNAMIC_COST, i),
        };
        
        let card = Card {
            id: object::new(ctx),
            card_type,
            level: synthesis_card_level,
            owner: sender,
            created_at: now
        };

        event::emit(CardSynthesized{
            old_cards: vector[object::uid_to_address(&card1.id), object::uid_to_address(&card2.id), object::uid_to_address(&card3.id)],
            new_card: object::uid_to_address(&card.id),
            card_type: card_name,
            rarity: synthesis_card_rarity,
            level: synthesis_card_level,
            owner: sender,
            created_at: now
        });

        burn_card(card1, clock, ctx);
        burn_card(card2, clock, ctx);
        burn_card(card3, clock, ctx);

        // spend fragment
        let req = fragment::spend_fragments(
            fragments, 
            string::utf8(b"Combine card."),
            clock,
            ctx
        );
        fragment::confirm_request(store, req, ctx);
        transfer::public_transfer(card, sender);
    }

    /// 销毁卡牌 
    public fun burn_card(card: Card, clock: &Clock, _ctx: &mut TxContext){
        let Card { id, card_type: _, level: _, owner, created_at: _ } = card;
        
        let card_id = object::uid_to_address(&id);
        
        event::emit(CardBurned {
            card_id,
            owner,
            burned_at: clock::timestamp_ms(clock)
        });

        object::delete(id);
    }

    #[test_only]
    public fun burn_card_for_testing(card: Card, ctx: &mut TxContext): (u8, address) {
        let Card { id, card_type, level: _, owner, created_at: _ } = card;
        let rarity = card_type.rarity;
        object::delete(id);
        (rarity, owner)
    }

    //---------------------------------------------- Get functions ----------------------------------------------//
    /// 获取卡牌ID
    public fun get_id(card: &Card): &UID {
        &card.id
    }

    // Getters
    public fun get_name(card: &Card): String { card.card_type.name }
    public fun get_rarity(card: &Card): u8 { card.card_type.rarity }
    public fun get_level(card: &Card): u8 { card.level }
    public fun get_owner(card: &Card): address { card.owner }
    public fun get_image_url(card: &Card): String { card.card_type.image_url }

    //---------------------------------------------- Helper functions ----------------------------------------------//
    fun rarity_mapping(rarity: u8): u8 {
        if(rarity == RARITY_COMMON){
            1
        }else if(rarity == RARITY_UNCOMMON){
            2
        }else if(rarity == RARITY_RARE){
            3
        }else {
            5
        }
    }

    fun synthesis_level_output(
        total_card_level: u8,
        rand: u64,
    ): u8 {
        let card_level;
        if(total_card_level <= 2){
            if(rand <= 70){
                card_level = 0
            }else if(rand > 95){
                card_level = 2
            }else{
                card_level = 1
            }
        }else if(total_card_level == 3 || total_card_level == 4){
            if(rand <= 40){
                card_level = 0
            }else if(rand > 40 && rand <= 80){
                card_level = 1
            }else if(rand > 98){
                card_level = 3
            }else{
                card_level = 2
            }
        }else if(total_card_level == 5 || total_card_level == 6){
            if(rand <= 15){
                card_level = 0
            }else if(rand > 15 && rand <= 60){
                card_level = 1
            }else if(rand > 90){
                card_level = 3
            }else{
                card_level = 2
            }
        }else if(total_card_level == 7 || total_card_level == 8){
            if(rand <= 5){
                card_level = 0
            }else if(rand > 5 && rand <= 30){
                card_level = 1
            }else if(rand > 75){
                card_level = 3
            }else{
                card_level = 2
            }
        }else{ 
            if(rand <= 15){
                card_level = 1
            }else if(rand > 70){
                card_level = 3
            }else{
                card_level = 2
            }
        };
        card_level
    }

    fun synthesis_rarity_output(
        total_card_rarity: u8,
        rand: u64,
    ): u8 {
        let card_rarity;
        if(total_card_rarity <= 5){
            if(rand <= 90){
                card_rarity = RARITY_COMMON
            }else if(rand > 99){
                card_rarity = RARITY_RARE
            }else{
                card_rarity = RARITY_UNCOMMON
            }
        }else if(total_card_rarity >= 6 && total_card_rarity <= 8){
            if(rand <= 60){
                card_rarity = RARITY_COMMON
            }else if(rand > 60 && rand <= 90){
                card_rarity = RARITY_UNCOMMON
            }else if(rand > 99){
                card_rarity = RARITY_LEGENDARY
            }else{
                card_rarity = RARITY_RARE
            }
        }else if(total_card_rarity >= 9 && total_card_rarity <= 11){
            if(rand <= 40){
                card_rarity = RARITY_COMMON
            }else if(rand > 40 && rand <= 80){
                card_rarity = RARITY_UNCOMMON
            }else if(rand > 98){
                card_rarity = RARITY_LEGENDARY
            }else{
                card_rarity = RARITY_RARE
            }
        }else if(total_card_rarity == 12 || total_card_rarity == 13){
            if(rand <= 20){
                card_rarity = RARITY_COMMON
            }else if(rand > 20 && rand <= 65){
                card_rarity = RARITY_UNCOMMON
            }else if(rand > 95){
                card_rarity = RARITY_LEGENDARY
            }else{
                card_rarity = RARITY_RARE
            }
        }else{ 
            if(rand <= 5){
                card_rarity = RARITY_COMMON
            }else if(rand > 5 && rand <= 35){
                card_rarity = RARITY_UNCOMMON
            }else if(rand > 75){
                card_rarity = RARITY_LEGENDARY
            }else{
                card_rarity = RARITY_RARE
            }
        };
        card_rarity
    }

    fun get_vector_id(
        rarity: u8,
        r: &Random,
        ctx: &mut TxContext
    ): u64{
        let card_id;
        if(rarity == RARITY_COMMON){
            let dice = generate_u64_in_range(&mut new_generator(r, ctx), 0, 1);
            if(dice == 0){
                card_id = 5;
            }else{
                card_id = 4;
            }
        }else if(rarity == RARITY_UNCOMMON){
            let dice = generate_u64_in_range(&mut new_generator(r, ctx), 0, 1);
            if(dice == 0){
                card_id = 1;
            }else{
                card_id = 2;
            }
        }else if(rarity == RARITY_RARE){
            let dice = generate_u64_in_range(&mut new_generator(r, ctx), 0, 2);
            if(dice == 0){
                card_id = 6;
            }else if(dice == 1){
                card_id = 3;
            }else{
                card_id = 0;
            }
        }else{
            card_id = 7;
        };
        card_id
    }
} 