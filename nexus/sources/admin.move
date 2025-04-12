module nexus::admin {
    use sui::event;

    /// 管理员权限凭证
    public struct AdminCap has key, store {
        id: UID
    }

    /// 管理员权限转移事件
    public struct AdminCapTransferred has copy, drop {
        cap_id: address,
        from: address,
        to: address
    }

    //---------------------------------------------- Init ----------------------------------------------//
    fun init(ctx: &mut TxContext) {
        // 创建管理员凭证并转移给部署者
        let admin_cap = AdminCap { id: object::new(ctx) };
        let sender = tx_context::sender(ctx);
        
        event::emit(AdminCapTransferred {
            cap_id: object::uid_to_address(&admin_cap.id),
            from: @0x0,
            to: sender
        });
        
        transfer::transfer(admin_cap, sender);
    }

    #[test_only]
    public fun init_for_testing(ctx: &mut TxContext) {
        init(ctx)
    }

    //---------------------------------------------- Entry functions ----------------------------------------------//
    public entry fun transfer_admin_cap(
        admin_cap: AdminCap,
        new_admin: address,
        ctx: &TxContext
    ) {
        let sender = tx_context::sender(ctx);
        
        event::emit(AdminCapTransferred {
            cap_id: object::uid_to_address(&admin_cap.id),
            from: sender,
            to: new_admin
        });
        
        transfer::public_transfer(admin_cap, new_admin);
    }
}