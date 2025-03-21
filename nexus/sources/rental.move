module nexus::rental {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;
    use sui::clock::{Self, Clock};
    use sui::table::{Self, Table};
    use sui::coin::{Self, Coin};
    use std::string::{Self, String};
    use nexus::card::{Self, Card};
    use nexus::fragment::{Self, FRAGMENT};
    use nexus::passport::{Self, Passport};

    /// 错误代码
    const EInvalidRentalPeriod: u64 = 0;
    const EInvalidUses: u64 = 1;
    const ENotOwner: u64 = 2;
    const ERentalExpired: u64 = 3;
    const ENoUsesLeft: u64 = 4;
    const EInsufficientPayment: u64 = 5;
    const ERentalNotExpired: u64 = 6;

    /// 租赁卡牌
    struct RentalCard has key {
        id: UID,
        original_card_id: address,
        owner: address,
        renter: address,
        rental_period: u64,
        uses_left: u64,
        expires_at: u64,
        rental_fee: u64,
        created_at: u64
    }

    /// 租赁相关事件
    struct CardRented has copy, drop {
        rental_id: address,
        card_id: address,
        owner: address,
        renter: address,
        rental_period: u64,
        uses: u64,
        fee: u64,
        rented_at: u64
    }

    struct RentalExpired has copy, drop {
        rental_id: address,
        card_id: address,
        owner: address,
        renter: address,
        expired_at: u64
    }

    struct RentalUsed has copy, drop {
        rental_id: address,
        card_id: address,
        renter: address,
        uses_left: u64,
        used_at: u64
    }

    /// 创建租赁卡牌
    public fun create_rental(
        original_card_id: address,
        rental_period: u64,
        uses: u64,
        rental_fee: u64,
        clock: &Clock,
        ctx: &mut TxContext
    ): RentalCard {
        assert!(rental_period > 0 && rental_period <= 30, EInvalidRentalPeriod); // 最长30天
        assert!(uses > 0 && uses <= 100, EInvalidUses); // 最多100次使用

        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);

        RentalCard {
            id: object::new(ctx),
            original_card_id,
            owner: sender,
            renter: @0x0, // 初始无租用者
            rental_period,
            uses_left: uses,
            expires_at: 0, // 初始未设置过期时间
            rental_fee,
            created_at: now
        }
    }

    /// 租用卡牌
    public fun rent_card(
        rental: &mut RentalCard,
        payment: Coin<FRAGMENT>,
        passport: &mut Passport,
        clock: &Clock,
        owner: address,
        ctx: &mut TxContext
    ) {
        let renter = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        
        // 检查支付金额
        assert!(coin::value(&payment) >= rental.rental_fee, EInsufficientPayment);
        
        // 更新租赁状态
        rental.renter = renter;
        rental.expires_at = now + (rental.rental_period * 24 * 60 * 60 * 1000); // 转换为毫秒
        
        // 添加到护照中
        passport::add_rental_card(passport, object::uid_to_address(&rental.id));

        event::emit(CardRented {
            rental_id: object::uid_to_address(&rental.id),
            card_id: rental.original_card_id,
            owner: rental.owner,
            renter,
            rental_period: rental.rental_period,
            uses: rental.uses_left,
            fee: rental.rental_fee,
            rented_at: now
        });

        // 转移租金给卡牌所有者
        transfer::public_transfer(payment, owner);
    }

    /// 使用租赁卡牌
    public fun use_rental(
        rental: &mut RentalCard,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let now = clock::timestamp_ms(clock);
        assert!(now <= rental.expires_at, ERentalExpired);
        assert!(rental.uses_left > 0, ENoUsesLeft);
        
        rental.uses_left = rental.uses_left - 1;

        event::emit(RentalUsed {
            rental_id: object::uid_to_address(&rental.id),
            card_id: rental.original_card_id,
            renter: rental.renter,
            uses_left: rental.uses_left,
            used_at: now
        });
    }

    /// 结束租赁
    public fun end_rental(
        rental: &mut RentalCard,
        passport: &mut Passport,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let now = clock::timestamp_ms(clock);
        assert!(now > rental.expires_at || rental.uses_left == 0, ERentalNotExpired);

        let renter = rental.renter;
        // 从护照中移除
        passport::remove_rental_card(passport, object::uid_to_address(&rental.id));

        event::emit(RentalExpired {
            rental_id: object::uid_to_address(&rental.id),
            card_id: rental.original_card_id,
            owner: rental.owner,
            renter,
            expired_at: now
        });

        // 重置租赁状态
        rental.renter = @0x0;
        rental.expires_at = 0;
    }

    // Getters
    public fun get_owner(rental: &RentalCard): address { rental.owner }
    public fun get_renter(rental: &RentalCard): address { rental.renter }
    public fun get_uses_left(rental: &RentalCard): u64 { rental.uses_left }
    public fun get_expires_at(rental: &RentalCard): u64 { rental.expires_at }
    public fun get_rental_fee(rental: &RentalCard): u64 { rental.rental_fee }
    public fun is_active(rental: &RentalCard, clock: &Clock): bool { 
        clock::timestamp_ms(clock) <= rental.expires_at && rental.uses_left > 0 
    }
} 