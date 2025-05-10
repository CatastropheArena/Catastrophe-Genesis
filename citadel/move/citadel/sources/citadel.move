/// Module: citadel
/// 链上游戏平台核心模块，负责管理用户、卡牌、游戏匹配等功能
module citadel::citadel {
    use sui::table::{Self, Table};
    use sui::clock::{Self, Clock};
    use sui::event;
    use std::string::{Self, String};
    use sui::object::{Self, UID, ID};
    use sui::tx_context::{Self, TxContext};
    use sui::transfer;
    use sui::vec_map::{Self, VecMap};
    use sui::vec_set::{Self, VecSet};
    use std::vector;
    use std::option::{Self, Option};
    use std::hash;
    use std::bcs;
    use sui::display;
    use sui::package;
    
    // 导入 nexus 模块
    use nexus::passport::{Self, Passport};
    use nexus::game::{Self, GameEntry};

    /// 错误码定义
    const EProfileAlreadyRegistered: u64 = 0;
    const EInvalidProfileName: u64 = 1;
    const EInvalidProfile: u64 = 2;
    const ENoAccess: u64 = 1;
    const EInvalidUsername: u64 = 2;
    const ENotEnoughPlayers: u64 = 6;
    const EInvalidAction: u64 = 7;
    const ELobbyFull: u64 = 10;
    const ENotLobbyLeader: u64 = 11;
    const ENotAuthorized: u64 = 12;
    const EGameEntryInvalid: u64 = 13;
    
    /// 常量定义
    const INIT_RATING: u64 = 1000;
    const MAX_PLAYERS_PER_LOBBY: u64 = 5;
    const MIN_PLAYERS_PER_MATCH: u64 = 2;
    const FRIEND_REQUEST_PENDING: u8 = 1;
    const FRIEND_REQUEST_ACCEPTED: u8 = 2;
    const DAILY_REWARDS_AMOUNT: u64 = 5;
    const DAY_IN_MS: u64 = 86400000; // 24小时 (毫秒)
    
    /// 用户状态
    const USER_STATUS_ONLINE: u8 = 1;
    const USER_STATUS_IN_LOBBY: u8 = 4;
    
    /// 游戏状态
    const MATCH_STATE_WAITING: u8 = 1;
    
    // ============= 全局存储 =============
    
    /// 游戏全局管理器
    public struct ManagerStore has key {
        id: UID,
        profiles: Table<address, address>, // Passport到Profile的映射
        ongoing_matches: vector<ID>,
        match_count: u64,
        lobby_count: u64,
    }

    // ============= 用户管理相关数据结构 =============
    
    /// 用户资料
    public struct Profile has key, store {
        id: UID,
        avatar: String,
        rating: u64,
        played: u64,
        won: u64,
        lost:u64
    }
    /// 用户注册事件
    public struct ProfileRegistered has copy, drop {
        profile_id: address,
        passport_id: address,
        sender: address,
    }
    /// 用户修改事件
    public struct ProfileModified has copy, drop {
        profile_id: address,
        avatar: String,
    }
    public struct ProfileStateUpdated has copy, drop {
        profile_id: address,
        rating: u64,
        played: u64,
        won: u64,
        lost: u64,
    }

    /// 用户好友关系
    public  struct FriendRelation has store {
        user_id: address,
        friend_id: address,
        status: u8, // 1=待确认, 2=已接受
        created_at: u64,
    }
    
    /// 好友关系存储
    public struct FriendshipStore has key {
        id: UID,
        relations: Table<address, vector<FriendRelation>>,
    }
    
    // ============= 卡牌相关数据结构 =============
    
    /// 卡牌类型枚举 (对应前端的CardName)
    public struct CardType has copy, drop, store {
        id: u8,
        name: String,
    }
    
    /// 实际卡牌
    public struct Card has key, store {
        id: UID,
        card_type: CardType,
        owner: address,
    }
    
    /// 游戏中的卡组
    public struct Deck has store {
        cards: vector<CardType>,
        discard: vector<CardType>,
    }

    // ============= 游戏匹配相关数据结构 =============
    
    /// 大厅模式
    public struct LobbyMode has store, copy, drop {
        mode_type: u8, // 1=default, 2=core, 3=random, 4=custom
        disabled_cards: vector<u8>, // 禁用卡牌类型ID列表
    }
    
    /// 游戏大厅
    public struct Lobby has key {
        id: UID,
        leader: address,
        participants: vector<address>, // 所有玩家
        spectators: vector<address>, // 观战者
        mode: LobbyMode,
        created_at: u64,
    }
    
    /// 游戏匹配队列
    public struct MatchQueue has key {
        id: UID,
        players: vector<address>,
        timestamp: u64,
    }
    
    /// 玩家在游戏中的状态
    public struct PlayerState has store {
        user_id: address,
        hand: vector<ID>, // 玩家手牌
        marked: vector<ID>, // 被标记的卡牌
        is_defeated: bool,
        defeat_reason: u8, // 0=未失败, 1=爆炸, 2=超时, 3=离开
        rating_before: u64,
        rating_change: u64,
    }
    
    /// 游戏对局
    public struct Match has key {
        id: UID,
        players: vector<PlayerState>,
        spectators: vector<address>,
        deck: Deck,
        draw_pile_size: u64,
        turn: u64, // 当前轮到第几个玩家
        state: u8, // 1=等待开始, 2=进行中, 3=已结束
        winner: Option<address>,
        turn_start_time: u64,
        match_start_time: u64,
        match_end_time: Option<u64>,
        is_reversed: bool, // 是否反转顺序
        attacks_remaining: u64, // 剩余攻击次数
        imploding_kitten_spot: Option<u64>, // 内爆猫在牌堆中的位置
    }
    
    /// 游戏对局历史记录
    public struct MatchHistory has key, store {
        id: UID,
        match_id: ID,
        players: vector<address>,
        winner: address,
        start_time: u64,
        end_time: u64,
        rating_changes: Table<address, u64>,
        player_results: Table<address, u8>, // 1=胜利, 2=失败
    }
    

    // ============= 事件定义 =============
    

    
    /// 用户登录事件
    public struct UserLoggedIn has copy, drop {
        user_id: address,
        username: String,
        timestamp: u64,
    }
    
    /// 好友请求事件
    public struct FriendRequestSent has copy, drop {
        from_user: address,
        to_user: address,
        timestamp: u64,
    }
    
    /// 好友接受事件
    public struct FriendRequestAccepted has copy, drop {
        from_user: address,
        to_user: address,
        timestamp: u64,
    }
    
    /// 大厅创建事件
    public struct LobbyCreated has copy, drop {
        lobby_id: ID,
        leader: address,
        timestamp: u64,
    }
    
    /// 玩家加入大厅事件
    public struct PlayerJoinedLobby has copy, drop {
        lobby_id: ID,
        player: address,
        timestamp: u64,
    }
    
    /// 游戏开始事件
    public struct MatchStarted has copy, drop {
        match_id: ID,
        players: vector<address>,
        timestamp: u64,
    }
    
    /// 玩家出牌事件
    public struct CardPlayed has copy, drop {
        match_id: ID,
        player: address,
        card_type: u8,
        timestamp: u64,
    }
    
    /// 游戏结束事件
    public struct MatchEnded has copy, drop {
        match_id: ID,
        winner: address,
        timestamp: u64,
    }
    
    /// 每日奖励领取事件
    public struct DailyRewardClaimed has copy, drop {
        user_id: address,
        amount: u64,
        timestamp: u64,
    }

    // ============= 管理员相关数据结构 =============
    
    /// 管理员权限凭证
    public struct AdminCap has key, store {
        id: UID,
        server_id: String,  // 服务器标识符
    }

    /// 管理员注册表
    public struct AdminRegistry has key {
        id: UID,
        admins: VecSet<address>, // 管理员地址集合
    }
    
    /// 游戏通行证关联
    public struct GameEntryPassport has store {
        game_entry_id: ID,     // nexus游戏通行证ID
        passport_id: address,  // nexus护照ID
        created_at: u64,
    }
    
    /// 游戏通行证存储
    public struct GameEntryStore has key {
        id: UID,
        entries: Table<ID, GameEntryPassport>, // 游戏通行证ID到护照ID的映射
    }
    
    /// 管理员添加事件
    public struct AdminAdded has copy, drop {
        admin_id: address,
        server_id: String,
        timestamp: u64,
    }
    
    /// 游戏通行证关联事件
    public struct GameEntryLinked has copy, drop {
        game_entry_id: ID,
        passport_id: address,
        timestamp: u64,
    }
    
    /// 玩家匹配队列事件
    public struct PlayerQueuedForMatch has copy, drop {
        player_passport: address,
        game_entry_id: ID,
        timestamp: u64,
    }
    
    /// 管理员撤销事件
    public struct AdminRevoked has copy, drop {
        admin_id: address,
        revoked_by: address,
        timestamp: u64,
    }
    public struct CITADEL has drop {}
    // ============= 初始化函数 =============
    
    fun init(witness: CITADEL,ctx: &mut TxContext) {
        let publisher = package::claim(witness, ctx);
        let keys = vector[
            std::string::utf8(b"rating"),
            std::string::utf8(b"played"),
            std::string::utf8(b"image_url"),
            std::string::utf8(b"project_url"),
        ];
        let values = vector[
            std::string::utf8(b"{rating}"),
            std::string::utf8(b"{played}"),
            std::string::utf8(b"{avatar}"),
            std::string::utf8(b"https://docs.walrus.site"),
        ];
        let mut display = display::new_with_fields<Profile>(&publisher, keys, values, ctx);
        display::update_version<Profile>(&mut display);

        let manager = ManagerStore {
            id: object::new(ctx),
            profiles: table::new(ctx),
            ongoing_matches: vector::empty(),
            match_count: 0,
            lobby_count: 0,
        };
        
        let friendship_store = FriendshipStore {
            id: object::new(ctx),
            relations: table::new(ctx),
        };
        
        let match_queue = MatchQueue {
            id: object::new(ctx),
            players: vector::empty(),
            timestamp: 0,
        };
        
        // 创建管理员注册表
        let mut admin_registry = AdminRegistry {
            id: object::new(ctx),
            admins: vec_set::empty(),
        };
        
        // 创建游戏通行证存储
        let game_entry_store = GameEntryStore {
            id: object::new(ctx),
            entries: table::new(ctx),
        };
        
        // 创建主管理员凭证
        let deployer = tx_context::sender(ctx);
        let admin_cap = AdminCap {
            id: object::new(ctx),
            server_id: string::utf8(b"main-server"),
        };
        
        // 添加部署者为管理员
        vec_set::insert(&mut admin_registry.admins, deployer);
        
        // 将全局资源共享出去
        transfer::share_object(manager);
        transfer::share_object(friendship_store);
        transfer::share_object(match_queue);
        transfer::share_object(admin_registry);
        transfer::share_object(game_entry_store);
        
        // 将管理员凭证转移给部署者
        transfer::public_transfer(publisher, ctx.sender());
        transfer::public_transfer(display, ctx.sender());
        transfer::transfer(admin_cap, deployer);
    }
    // ============= Profile管理函数 =============
    /// 用户使用护照创建Profile
    public entry fun create_profile_with_passport(
        manager: &mut ManagerStore,
        friendship: &mut FriendshipStore,
        passport: &Passport,
        avatar: String,
        ctx: &mut TxContext
    ) {
        // 获取护照ID
        let passport_id = passport.get_passport_id();
        
        // 检查关联的Profile是否已存在
        assert!(!table::contains(&manager.profiles, passport_id), EProfileAlreadyRegistered);
   
        // 创建用户对象
        let profile = Profile {
            id: object::new(ctx),
            avatar,
            rating: INIT_RATING, // 初始分数
            played: 0,
            won: 0,
            lost: 0,
        };
        
        let profile_id = object::uid_to_address(&profile.id);
        
        // 更新管理器
        table::add(&mut manager.profiles, passport_id, profile_id);        
        // 初始化好友关系存储
        table::add(&mut friendship.relations, profile_id, vector::empty<FriendRelation>());
        

        // 发送注册事件
        let sender = tx_context::sender(ctx);
        event::emit(ProfileRegistered {
            profile_id,
            passport_id,
            sender,
        });
        
        // 将profile对象共享给全局
        transfer::share_object(profile);
    }
    /// 用户修改Profile
    public entry fun modify_profile(
        manager: &mut ManagerStore,
        profile: &mut Profile,
        passport: &Passport,
        avatar: String
    ) {
        let passport_id = passport.get_passport_id();
        let profile_id = table::borrow(&manager.profiles, passport_id);
        // 判断通过passport_id获取的profile_id是否等于profile的id
        assert!(profile_id == profile.id.to_address(), EInvalidProfile);
        profile.avatar = avatar;
        event::emit(ProfileModified {
            profile_id: profile.id.to_address(),
            avatar,
        });
    }

    /// 管理员为某个passport_id创建Profile
    public entry fun create_profile_for_passport(
        manager: &mut ManagerStore,
        friendship: &mut FriendshipStore,
        passport_id: address,
        avatar: String,
        _: &AdminCap,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        // 检查用户名是否已存在
        assert!(!table::contains(&manager.profiles, passport_id), EProfileAlreadyRegistered);
        
        // 检查用户名是否有效
        
        // 创建用户对象
        let profile = Profile {
            id: object::new(ctx),
            avatar,
            rating: INIT_RATING, // 初始分数
            played: 0,
            won: 0,
            lost: 0,
        };
        
        let profile_id = object::uid_to_address(&profile.id);
        
        // 更新管理器
        table::add(&mut manager.profiles, passport_id, profile_id);
        
        // 初始化好友关系存储
        table::add(&mut friendship.relations, profile_id, vector::empty<FriendRelation>());
        
        // 发送注册事件
        event::emit(ProfileRegistered {
            profile_id,
            passport_id,
            sender,
        });
        
        // 将profile对象共享给全局
        transfer::share_object(profile);
    }

    /// 管理员修改某个Profile的基础数据
    public entry fun modify_profile_data(
        profile: &mut Profile,
        avatar: String,
        _: &AdminCap,
    ) {
        profile.avatar = avatar;
        event::emit(ProfileModified {
            profile_id: profile.id.to_address(),
            avatar,
        });
    }
    
    /// 某个Profile胜利
    public entry fun profile_win(
        profile: &mut Profile,
        _: &AdminCap,
    ) {
        profile.won = profile.won + 1;
        profile.played = profile.played + 1;
        event::emit(ProfileStateUpdated {
            profile_id: profile.id.to_address(),
            rating: profile.rating,
            played: profile.played,
            won: profile.won,
            lost: profile.lost,
        });
    }

    /// 某个Profile失败
    public entry fun profile_lose(
        profile: &mut Profile,
        _: &AdminCap,
    ) {
        profile.lost = profile.lost + 1;
        profile.played = profile.played + 1;
        event::emit(ProfileStateUpdated {
            profile_id: profile.id.to_address(),
            rating: profile.rating,
            played: profile.played,
            won: profile.won,
            lost: profile.lost,
        });
    }

    /// 修改Profile的分数
    public entry fun update_profile_rating(
        profile: &mut Profile,
        rating: u64,
        _: &AdminCap,
    ) {
        profile.rating = rating;
        event::emit(ProfileStateUpdated {
            profile_id: profile.id.to_address(),
            rating,
            played: profile.played,
            won: profile.won,
            lost: profile.lost,
        });
    }

    /// 获取用户分数
    public fun get_user_rating(user: &Profile): u64 {
        user.rating
    }
    
    /// 计算胜率
    public fun get_winrate(user: &Profile): u64 {
        if (user.played == 0) {
            return 0
        };
        
        (user.won * 100) / user.played
    }

    // ============= Nexus集成函数 =============
    
    /// 校验用户是否具有Nexus通行证
    public fun verify_nexus_passport(
        passport: &Passport,
        game_entry: &GameEntry
    ): bool {
        // 检查Nexus通行证是否属于同一个护照
        let passport_id =&passport.get_passport_id() ;
        let entry_passport_id = &game_entry.get_passport_id();
        passport_id == entry_passport_id
    }

    /// 检查调用者是否有权限访问特定ID的入口函数
    entry fun seal_approve_verify_nexus_passport(
        id: vector<u8>,
        passport: &Passport,
        game_entry: &GameEntry,
     ) {
        let passport_id = bcs::to_bytes(&passport.get_passport_id());
        assert!(id == passport_id, ENoAccess);
        assert!(verify_nexus_passport(passport, game_entry), ENoAccess);
    }

    /// 发送好友请求
    public entry fun send_friend_request(
        friendship_store: &mut FriendshipStore,
        to_user_id: address,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        
        // 添加到发送者的关系列表
        let sender_relations = table::borrow_mut(&mut friendship_store.relations, sender);
        
        let relation = FriendRelation {
            user_id: sender,
            friend_id: to_user_id,
            status: FRIEND_REQUEST_PENDING,
            created_at: now,
        };
        
        vector::push_back(sender_relations, relation);
        
        // 添加到接收者的关系列表
        let receiver_relations = table::borrow_mut(&mut friendship_store.relations, to_user_id);
        
        let relation = FriendRelation {
            user_id: to_user_id,
            friend_id: sender,
            status: FRIEND_REQUEST_PENDING,
            created_at: now,
        };
        
        vector::push_back(receiver_relations, relation);
        
        // 发送事件
        event::emit(FriendRequestSent {
            from_user: sender,
            to_user: to_user_id,
            timestamp: now,
        });
    }
    
    /// 接受好友请求
    public entry fun accept_friend_request(
        friendship_store: &mut FriendshipStore,
        from_user_id: address,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let receiver = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        
        // 更新接收者的关系状态
        let receiver_relations = table::borrow_mut(&mut friendship_store.relations, receiver);
        let receiver_relations_len = vector::length(receiver_relations);
        let mut i = 0;
        
        while (i < receiver_relations_len) {
            let relation = vector::borrow_mut(receiver_relations, i);
            if (relation.friend_id == from_user_id) {
                relation.status = FRIEND_REQUEST_ACCEPTED;
                break;
            };
            i = i + 1;
        };
        
        // 更新发送者的关系状态
        let sender_relations = table::borrow_mut(&mut friendship_store.relations, from_user_id);
        let sender_relations_len = vector::length(sender_relations);
        let mut i = 0;
        
        while (i < sender_relations_len) {
            let relation = vector::borrow_mut(sender_relations, i);
            if (relation.friend_id == receiver) {
                relation.status = FRIEND_REQUEST_ACCEPTED;
                break;
            };
            i = i + 1;
        };
        
        // 发送事件
        event::emit(FriendRequestAccepted {
            from_user: from_user_id,
            to_user: receiver,
            timestamp: now,
        });
    }
    
    
    // ============= 游戏大厅函数 =============
    
    /// 创建游戏大厅
    public entry fun create_lobby(
        manager: &mut ManagerStore,
        user: &mut Profile,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        
        // 创建默认大厅模式
        let mode = LobbyMode {
            mode_type: 1, // default
            disabled_cards: vector::empty(),
        };
        
        // 创建大厅
        let lobby = Lobby {
            id: object::new(ctx),
            leader: sender,
            participants: vector::singleton(sender),
            spectators: vector::empty(),
            mode,
            created_at: now,
        };
        
        let lobby_id = object::uid_to_address(&lobby.id);
                
        // 更新管理器
        manager.lobby_count = manager.lobby_count + 1;
        
        // 发送事件
        event::emit(LobbyCreated {
            lobby_id: object::id(&lobby),
            leader: sender,
            timestamp: now,
        });
        
        // 共享大厅对象
        transfer::share_object(lobby);
    }
    
    /// 加入游戏大厅
    public entry fun join_lobby(
        user: &mut Profile,
        lobby: &mut Lobby,
        as_spectator: bool,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        
        if (as_spectator) {
            // 加入为观战者
            assert!(!vector::contains(&lobby.spectators, &sender), EInvalidAction);
            vector::push_back(&mut lobby.spectators, sender);
        } else {
            // 加入为玩家
            assert!(!vector::contains(&lobby.participants, &sender), EInvalidAction);
            assert!(vector::length(&lobby.participants) < MAX_PLAYERS_PER_LOBBY, ELobbyFull);
            vector::push_back(&mut lobby.participants, sender);
        };
        
        
        // 发送事件
        event::emit(PlayerJoinedLobby {
            lobby_id: object::id(lobby),
            player: sender,
            timestamp: now,
        });
    }
    
    /// 开始游戏
    public entry fun start_match(
        manager: &mut ManagerStore,
        lobby: &mut Lobby,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        let sender = tx_context::sender(ctx);
        let now = clock::timestamp_ms(clock);
        
        // 检查是否是大厅领导者
        assert!(sender == lobby.leader, ENotLobbyLeader);
        
        // 检查人数是否足够
        assert!(vector::length(&lobby.participants) >= MIN_PLAYERS_PER_MATCH, ENotEnoughPlayers);
        
        // 创建游戏对局
        let match_obj = create_match_from_lobby(lobby, clock, ctx);
        let match_id = object::id(&match_obj);
        
        // 更新管理器
        vector::push_back(&mut manager.ongoing_matches, match_id);
        manager.match_count = manager.match_count + 1;
        
        // 发送事件
        event::emit(MatchStarted {
            match_id,
            players: lobby.participants,
            timestamp: now,
        });
        
        // 共享游戏对局
        transfer::share_object(match_obj);
    }
    
    /// 内部函数：从大厅创建游戏对局
    fun create_match_from_lobby(
        lobby: &Lobby,
        clock: &Clock,
        ctx: &mut TxContext
    ): Match {
        let now = clock::timestamp_ms(clock);
        
        // 创建玩家状态列表
        let mut players = vector::empty<PlayerState>();
        let participant_count = vector::length(&lobby.participants);
        
        let mut i = 0;
        while (i < participant_count) {
            let player_id = *vector::borrow(&lobby.participants, i);
            
            let player_state = PlayerState {
                user_id: player_id,
                hand: vector::empty(), // 稍后分配卡牌
                marked: vector::empty(),
                is_defeated: false,
                defeat_reason: 0,
                rating_before: 1000, // 将在游戏开始时从用户对象获取实际的分数
                rating_change: 0,
            };
            
            vector::push_back(&mut players, player_state);
            i = i + 1;
        };
        
        // 创建游戏牌组
        let deck = create_initial_deck(ctx);
        
        // 创建游戏对局
        Match {
            id: object::new(ctx),
            players,
            spectators: lobby.spectators,
            deck,
            draw_pile_size: 0, // 将在洗牌后设置
            turn: 0, // 第一个玩家
            state: MATCH_STATE_WAITING,
            winner: option::none(),
            turn_start_time: now,
            match_start_time: now,
            match_end_time: option::none(),
            is_reversed: false,
            attacks_remaining: 0,
            imploding_kitten_spot: option::none(),
        }
    }
    
    /// 创建初始卡组
    fun create_initial_deck(_: &TxContext): Deck {
        // 这里简化处理，实际实现应该根据游戏规则创建完整的卡组
        let mut  cards = vector::empty<CardType>();
        
        // 添加基本卡牌
        // 爆炸猫
        add_card_type(&mut cards, 1, string::utf8(b"exploding-kitten"));
        
        // 拆除
        add_card_type(&mut cards, 2, string::utf8(b"defuse"));
        
        // 攻击
        add_card_type(&mut cards, 3, string::utf8(b"attack"));
        
        // 否决
        add_card_type(&mut cards, 4, string::utf8(b"nope"));
        
        // 洗牌
        add_card_type(&mut cards, 5, string::utf8(b"shuffle"));
        
        // 跳过
        add_card_type(&mut cards, 6, string::utf8(b"skip"));
        
        // 偷看
        add_card_type(&mut cards, 7, string::utf8(b"see-the-future-3x"));
        
        Deck {
            cards,
            discard: vector::empty(),
        }
    }
    
    /// 添加卡牌类型到卡组
    fun add_card_type(cards: &mut vector<CardType>, id: u8, name: String) {
        let card_type = CardType {
            id,
            name,
        };
        vector::push_back(cards, card_type);
    }
    

    // ============= 管理员功能 =============
    
    /// 创建新的管理员凭证
    public entry fun create_admin(
        admin_registry: &mut AdminRegistry,
        admin_cap: &AdminCap,
        new_admin: address,
        server_id: String,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 创建新的管理员凭证
        let new_admin_cap = AdminCap {
            id: object::new(ctx),
            server_id,
        };
        
        // 添加到管理员注册表
        if (!vec_set::contains(&admin_registry.admins, &new_admin)) {
            vec_set::insert(&mut admin_registry.admins, new_admin);
        };
        
        // 发送事件
        event::emit(AdminAdded {
            admin_id: new_admin,
            server_id,
            timestamp: clock::timestamp_ms(clock),
        });
        
        // 转移管理员凭证
        transfer::transfer(new_admin_cap, new_admin);
    }
    
    /// 关联游戏通行证与护照
    public entry fun link_game_entry(
        game_entry_store: &mut GameEntryStore,
        admin_registry: &AdminRegistry,
        game_entry_id: ID,
        passport_id: address,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        let now = clock::timestamp_ms(clock);
        
        // 创建游戏通行证关联
        let entry_passport = GameEntryPassport {
            game_entry_id,
            passport_id,
            created_at: now,
        };
        
        // 添加到存储
        table::add(&mut game_entry_store.entries, game_entry_id, entry_passport);
        
        // 发送事件
        event::emit(GameEntryLinked {
            game_entry_id,
            passport_id,
            timestamp: now,
        });
    }
    
    /// 将玩家添加到匹配队列
    public entry fun add_to_match_queue(
        match_queue: &mut MatchQueue,
        game_entry_store: &GameEntryStore,
        admin_registry: &AdminRegistry,
        game_entry_id: ID,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 检查游戏通行证是否存在
        assert!(table::contains(&game_entry_store.entries, game_entry_id), EGameEntryInvalid);
        
        let entry_passport = table::borrow(&game_entry_store.entries, game_entry_id);
        let passport_id = entry_passport.passport_id;
        
        // 将玩家添加到匹配队列
        vector::push_back(&mut match_queue.players, passport_id);
        match_queue.timestamp = clock::timestamp_ms(clock);
        
        // 发送事件
        event::emit(PlayerQueuedForMatch {
            player_passport: passport_id,
            game_entry_id,
            timestamp: match_queue.timestamp,
        });
    }
    

    // ============= 游戏匹配函数 =============
    
    /// 从匹配队列创建游戏对局（仅管理员）
    public entry fun create_match_from_queue(
        manager: &mut ManagerStore,
        match_queue: &mut MatchQueue,
        admin_registry: &AdminRegistry,
        min_players: u64,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 检查队列中的玩家是否足够
        let queue_size = vector::length(&match_queue.players);
        assert!(queue_size >= min_players, ENotEnoughPlayers);
        
        let now = clock::timestamp_ms(clock);
        
        // 创建玩家状态列表
        let mut players = vector::empty<PlayerState>();
        let mut player_addresses = vector::empty<address>();
        
        // 从队列中选择玩家
        let mut i = 0;
        while (i < min_players) {
            let player_id = vector::borrow(&match_queue.players, i);
            
            let player_state = PlayerState {
                user_id: *player_id,
                hand: vector::empty(), // 稍后分配卡牌
                marked: vector::empty(),
                is_defeated: false,
                defeat_reason: 0,
                rating_before: 1000, // 将在游戏开始时获取实际的分数
                rating_change: 0,
            };
            
            vector::push_back(&mut players, player_state);
            vector::push_back(&mut player_addresses, *player_id);
            i = i + 1;
        };
        
        // 创建游戏牌组
        let deck = create_initial_deck(ctx);
        
        // 创建游戏对局
        let match_obj = Match {
            id: object::new(ctx),
            players,
            spectators: vector::empty(),
            deck,
            draw_pile_size: 0, // 将在洗牌后设置
            turn: 0, // 第一个玩家
            state: MATCH_STATE_WAITING,
            winner: option::none(),
            turn_start_time: now,
            match_start_time: now,
            match_end_time: option::none(),
            is_reversed: false,
            attacks_remaining: 0,
            imploding_kitten_spot: option::none(),
        };
        
        let match_id = object::id(&match_obj);
        
        // 更新管理器
        vector::push_back(&mut manager.ongoing_matches, match_id);
        manager.match_count = manager.match_count + 1;
        
        // 发送事件
        event::emit(MatchStarted {
            match_id,
            players: player_addresses,
            timestamp: now,
        });
        
        // 清空队列（移除已匹配的玩家）
        if (queue_size == min_players) {
            match_queue.players = vector::empty();
        } else {
            // 保留剩余玩家
            let mut i = 0;
            while (i < min_players) {
                vector::remove(&mut match_queue.players, 0);
                i = i + 1;
            };
        };
        
        // 共享游戏对局
        transfer::share_object(match_obj);
    }
    
    /// 更新游戏状态（仅管理员）
    public entry fun update_match_state(
        match_obj: &mut Match,
        admin_registry: &AdminRegistry,
        state: u8,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 更新游戏状态
        match_obj.state = state;
        
        // 如果游戏结束，记录结束时间
        if (state == 3) { // 已结束
            match_obj.match_end_time = option::some(clock::timestamp_ms(clock));
        };
    }
    
    /// 设置游戏胜利者（仅管理员）
    public entry fun set_match_winner(
        match_obj: &mut Match,
        admin_registry: &AdminRegistry,
        winner: address,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 更新胜利者
        match_obj.winner = option::some(winner);
    }
    
    /// 检查地址是否为管理员
    public fun is_admin(admin_registry: &AdminRegistry, addr: address): bool {
        vec_set::contains(&admin_registry.admins, &addr)
    }
    
    /// 获取游戏通行证关联的护照ID
    public fun get_passport_from_game_entry(
        game_entry_store: &GameEntryStore,
        game_entry_id: ID
    ): address {
        let entry_passport = table::borrow(&game_entry_store.entries, game_entry_id);
        entry_passport.passport_id
    }
    
    /// 撤销管理员权限
    public entry fun revoke_admin(
        admin_registry: &mut AdminRegistry,
        admin_cap: &AdminCap,
        admin_to_revoke: address,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 防止撤销自己的权限
        assert!(admin_to_revoke != sender, EInvalidAction);
        
        // 检查要撤销的地址是否为管理员
        assert!(vec_set::contains(&admin_registry.admins, &admin_to_revoke), EInvalidAction);
        
        // 从管理员注册表中移除
        vec_set::remove(&mut admin_registry.admins, &admin_to_revoke);
        
        // 发送事件
        event::emit(AdminRevoked {
            admin_id: admin_to_revoke,
            revoked_by: sender,
            timestamp: clock::timestamp_ms(clock),
        });
    }

    /// 创建游戏历史记录（仅管理员）
    public entry fun create_match_history(
        admin_registry: &AdminRegistry,
        match_obj: &Match,
        winner: address,
        player_results: vector<address>, // 胜利的玩家地址列表
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 获取结束时间，如果未结束则使用当前时间
        let end_time = if (option::is_some(&match_obj.match_end_time)) {
            *option::borrow(&match_obj.match_end_time)
        } else {
            clock::timestamp_ms(clock)
        };
        
        // 创建玩家列表
        let mut players = vector::empty<address>();
        let players_count = vector::length(&match_obj.players);
        let mut i = 0;
        while (i < players_count) {
            let player_state = vector::borrow(&match_obj.players, i);
            vector::push_back(&mut players, player_state.user_id);
            i = i + 1;
        };
        
        // 创建评分变化和结果表
        let mut rating_changes = table::new<address, u64>(ctx);
        let mut player_results_table = table::new<address, u8>(ctx);
        
        // 填充结果表
        i = 0;
        while (i < players_count) {
            let player_id = *vector::borrow(&players, i);
            
            // 默认评分变化为0
            table::add(&mut rating_changes, player_id, 0);
            
            // 设置结果 (1=胜利, 2=失败)
            if (vector::contains(&player_results, &player_id)) {
                table::add(&mut player_results_table, player_id, 1);
            } else {
                table::add(&mut player_results_table, player_id, 2);
            };
            
            i = i + 1;
        };
        
        // 创建历史记录对象
        let history = MatchHistory {
            id: object::new(ctx),
            match_id: object::id(match_obj),
            players,
            winner,
            start_time: match_obj.match_start_time,
            end_time,
            rating_changes,
            player_results: player_results_table,
        };
        
        // 发送事件
        event::emit(MatchEnded {
            match_id: object::id(match_obj),
            winner,
            timestamp: end_time,
        });
        
        // 共享历史记录对象
        transfer::share_object(history);
    }

    /// 更新用户评分（仅管理员）
    public entry fun update_user_rating(
        admin_registry: &AdminRegistry,
        user: &mut Profile,
        match_history: &MatchHistory,
        rating_change: u64,
        is_win: bool,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 更新评分
        if (is_win) {
            user.rating = user.rating + rating_change;
            user.won = user.won + 1;
        } else {
            // 防止评分变为负数
            user.rating = if (user.rating > rating_change) {
                user.rating - rating_change
            } else {
                0
            };
        };
        
        // 更新总场次
        user.played = user.played + 1;
    }

    /// 为玩家分配卡牌（仅管理员）
    public entry fun assign_card_to_player(
        admin_registry: &AdminRegistry,
        match_obj: &mut Match,
        player_index: u64,
        card_type: u8,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 检查玩家索引是否有效
        assert!(player_index < vector::length(&match_obj.players), EInvalidAction);
        
        // 复制match_id (在创建事件前)
        let match_id = object::uid_to_inner(&match_obj.id);
        
        // 创建卡牌
        let card = Card {
            id: object::new(ctx),
            card_type: CardType {
                id: card_type,
                name: get_card_name_by_id(card_type),
            },
            owner: vector::borrow(&match_obj.players, player_index).user_id,
        };
        
        // 添加到玩家手牌
        let player_state = vector::borrow_mut(&mut match_obj.players, player_index);
        vector::push_back(&mut player_state.hand, object::id(&card));
        
        // 发送事件
        event::emit(CardPlayed {
            match_id,
            player: player_state.user_id,
            card_type,
            timestamp: tx_context::epoch_timestamp_ms(ctx),
        });
        
        // 转移卡牌给玩家
        transfer::transfer(card, player_state.user_id);
    }
    
    /// 玩家抽牌（仅管理员）
    public entry fun draw_card(
        admin_registry: &AdminRegistry,
        match_obj: &mut Match,
        player_index: u64,
        card_index: u64, // 由管理员指定要抽取的牌的索引
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 检查发送者是否为授权管理员
        let sender = tx_context::sender(ctx);
        assert!(vec_set::contains(&admin_registry.admins, &sender), ENotAuthorized);
        
        // 检查玩家索引是否有效
        assert!(player_index < vector::length(&match_obj.players), EInvalidAction);
        
        // 检查牌堆是否有牌
        assert!(match_obj.draw_pile_size > 0, EInvalidAction);
        
        // 检查卡牌索引是否有效
        assert!(card_index < vector::length(&match_obj.deck.cards), EInvalidAction);
        
        // 复制match_id (在创建事件前)
        let match_id = object::uid_to_inner(&match_obj.id);
        
        // 减少牌堆大小
        match_obj.draw_pile_size = match_obj.draw_pile_size - 1;
        
        // 获取指定的卡牌类型
        let card_type = vector::borrow(&match_obj.deck.cards, card_index).id;
        
        // 创建卡牌
        let card = Card {
            id: object::new(ctx),
            card_type: CardType {
                id: card_type,
                name: get_card_name_by_id(card_type),
            },
            owner: vector::borrow(&match_obj.players, player_index).user_id,
        };
        
        // 添加到玩家手牌
        let player_state = vector::borrow_mut(&mut match_obj.players, player_index);
        vector::push_back(&mut player_state.hand, object::id(&card));
        
        // 将牌从牌堆移到弃牌堆
        let card_type = *vector::borrow(&match_obj.deck.cards, card_index);
        vector::swap_remove(&mut match_obj.deck.cards, card_index);
        vector::push_back(&mut match_obj.deck.discard, card_type);
        
        // 发送事件
        event::emit(CardPlayed {
            match_id,
            player: player_state.user_id,
            card_type: card_type.id,
            timestamp: clock::timestamp_ms(clock),
        });
        
        // 转移卡牌给玩家
        transfer::transfer(card, player_state.user_id);
    }
    
    /// 根据ID获取卡牌名称
    fun get_card_name_by_id(id: u8): String {
        if (id == 1) {
            return string::utf8(b"exploding-kitten")
        } else if (id == 2) {
            return string::utf8(b"defuse")
        } else if (id == 3) {
            return string::utf8(b"attack")
        } else if (id == 4) {
            return string::utf8(b"nope")
        } else if (id == 5) {
            return string::utf8(b"shuffle")
        } else if (id == 6) {
            return string::utf8(b"skip")
        } else if (id == 7) {
            return string::utf8(b"see-the-future-3x")
        } else {
            return string::utf8(b"unknown")
        }
    }
    
    

    /// 使用游戏通行证加入匹配队列
    public entry fun join_match_queue_with_entry(
        match_queue: &mut MatchQueue,
        game_entry: &GameEntry,
        passport: &Passport,
        clock: &Clock,
        ctx: &mut TxContext
    ) {
        // 验证游戏通行证的有效性
        assert!(verify_nexus_passport( passport,game_entry), EGameEntryInvalid);
        
        // 获取护照ID
        let passport_id = passport::get_passport_id(passport);
        
        // 检查玩家是否已在队列中
        assert!(!vector::contains(&match_queue.players, &passport_id), EInvalidAction);
        
        // 添加到匹配队列
        vector::push_back(&mut match_queue.players, passport_id);
        match_queue.timestamp = clock::timestamp_ms(clock);
        
        // 发送事件
        event::emit(PlayerQueuedForMatch {
            player_passport: passport_id,
            game_entry_id: object::id(game_entry),
            timestamp: match_queue.timestamp,
        });
    }
    
    /// 获取玩家的护照信息
    public fun get_player_passport_info(
        passport: &Passport
    ): (address, u64) {
        // 返回护照ID和每日奖励领取次数（可以作为玩家活跃度参考）
        (
            passport::get_passport_id(passport),
            passport::get_daily_rewards_claimed(passport)
        )
    }
}
