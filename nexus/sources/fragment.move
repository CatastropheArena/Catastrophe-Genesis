module nexus::fragment {
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::event;
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin, TreasuryCap};
    use std::string::{Self, String};
    use sui::package;
    use std::option;

    /// 错误代码
    const EInsufficientFragments: u64 = 0;
    const ENotOwner: u64 = 1;

    /// One-time witness for the module
    struct FRAGMENT has drop {}

    /// Fragment coin metadata
    struct FragmentMetadata has key {
        id: UID
    }

    /// 碎片铸造事件
    struct FragmentMinted has copy, drop {
        amount: u64,
        recipient: address,
        minted_at: u64
    }

    /// 碎片销毁事件
    struct FragmentBurned has copy, drop {
        amount: u64,
        owner: address,
        burned_at: u64
    }

    /// 碎片转移事件
    struct FragmentTransferred has copy, drop {
        amount: u64,
        from: address,
        to: address,
        transferred_at: u64
    }

    /// 模块初始化
    fun init(witness: FRAGMENT, ctx: &mut TxContext) {
        // 创建碎片代币
        let (treasury_cap, metadata) = coin::create_currency(
            witness,
            9, // 精度
            b"FRAG",
            b"Fragment",
            b"Game card fragments for synthesis",
            option::none(),
            ctx
        );
        
        // 包装元数据对象
        let metadata_wrapper = FragmentMetadata {
            id: object::new(ctx)
        };
        
        // 发布元数据为不可变对象
        transfer::public_freeze_object(metadata);
        transfer::public_share_object(metadata_wrapper);
        
        // 转移铸币权给部署者
        transfer::public_transfer(treasury_cap, tx_context::sender(ctx));
    }

    /// 管理员铸造碎片
    public fun mint(
        treasury_cap: &mut TreasuryCap<FRAGMENT>, 
        amount: u64, 
        recipient: address, 
        ctx: &mut TxContext
    ) {
        let coin = coin::mint(treasury_cap, amount, ctx);
        
        event::emit(FragmentMinted {
            amount,
            recipient,
            minted_at: tx_context::epoch_timestamp_ms(ctx)
        });

        transfer::public_transfer(coin, recipient);
    }

    /// 销毁碎片
    public fun burn(
        treasury_cap: &mut TreasuryCap<FRAGMENT>,
        coin: Coin<FRAGMENT>, 
        ctx: &mut TxContext
    ) {
        let amount = coin::value(&coin);
        let owner = tx_context::sender(ctx);

        coin::burn(treasury_cap, coin);

        event::emit(FragmentBurned {
            amount,
            owner,
            burned_at: tx_context::epoch_timestamp_ms(ctx)
        });
    }

    /// 合并碎片
    public entry fun merge(coin1: &mut Coin<FRAGMENT>, coin2: Coin<FRAGMENT>) {
        coin::join(coin1, coin2);
    }

    /// 分割碎片
    public fun split(coin: &mut Coin<FRAGMENT>, amount: u64, ctx: &mut TxContext): Coin<FRAGMENT> {
        coin::split(coin, amount, ctx)
    }

    /// 转移碎片
    public entry fun transfer_fragments(coin: Coin<FRAGMENT>, recipient: address, ctx: &mut TxContext) {
        let amount = coin::value(&coin);
        let sender = tx_context::sender(ctx);

        event::emit(FragmentTransferred {
            amount,
            from: sender,
            to: recipient,
            transferred_at: tx_context::epoch_timestamp_ms(ctx)
        });

        transfer::public_transfer(coin, recipient);
    }

    /// 获取碎片余额
    public fun balance(coin: &Coin<FRAGMENT>): u64 {
        coin::value(coin)
    }
} 