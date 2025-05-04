# 对战模型子目录 (Match Model)

## 概述

对战模型子目录包含当前对战模块的核心状态管理逻辑和数据结构。它定义了整个对战过程中的状态管理架构，包括状态存储、操作定义、选择器和相关钩子。作为整个对战功能的基础，模型子目录为所有对战相关子模块提供了一致的状态管理基础，确保游戏状态的一致性、可预测性和可维护性。

## 文件结构

```
model/
├── index.ts        # 模型导出
├── store.ts        # Redux状态存储定义
├── actions.ts      # 对战核心操作
├── selectors.ts    # 状态选择器
└── hooks.ts        # 自定义React钩子
```

## 功能职责

### 核心状态定义

定义和管理对战的核心状态数据：
- 对战基本信息（ID、模式、创建时间等）
- 参与玩家状态（玩家信息、生命值、资源等）
- 游戏局面状态（场上卡牌、回合信息等）
- 对战进度状态（是否结束、胜者等）
- 对战设置和规则参数

### 状态更新操作

提供修改对战状态的操作集合：
- 初始化和启动对战
- 更新玩家和游戏状态
- 处理游戏行动和结果
- 同步服务器状态
- 处理对战结束逻辑

### 状态选择器

提供访问对战状态的标准方式：
- 获取当前对战信息
- 检索玩家状态数据
- 查询游戏局面信息
- 计算派生状态（如可用操作）
- 过滤和筛选状态数据

### 状态管理钩子

提供React组件使用的自定义钩子：
- 对战状态读取钩子
- 对战操作执行钩子
- 状态变化监听钩子
- 玩家操作辅助钩子
- 对战流程控制钩子

## 数据模型

对战模型子目录的核心数据结构：

```typescript
// 对战状态
export interface CurrentMatchState {
  matchId: string | null;
  gameMode: GameMode | null;
  startTime: number | null;
  players: Record<string, PlayerState>;
  currentPlayerId: string | null;
  opponentId: string | null;
  currentTurn: number;
  turnStartTime: number | null;
  phase: GamePhase | null;
  board: BoardState;
  isMatchLoading: boolean;
  isMatchEnded: boolean;
  winner: string | null;
  isSpectating: boolean;
  spectators: number;
  error: string | null;
}

// 玩家状态
export interface PlayerState {
  id: string;
  username: string;
  avatar: string;
  health: number;
  maxHealth: number;
  mana: number;
  maxMana: number;
  deckSize: number;
  handSize: number;
  hand: Card[];
  field: Card[];
  graveyard: Card[];
  effects: Effect[];
  isConnected: boolean;
  isReady: boolean;
}

// 游戏板状态
export interface BoardState {
  playerFieldCards: Card[];
  opponentFieldCards: Card[];
  activeEffects: Effect[];
  lastPlayedCard: Card | null;
  highlightedPositions: Position[];
  animationInProgress: boolean;
}

// 游戏模式枚举
export enum GameMode {
  CASUAL = 'CASUAL',
  RANKED = 'RANKED',
  DRAFT = 'DRAFT',
  SPECIAL_EVENT = 'SPECIAL_EVENT'
}

// 游戏阶段枚举
export enum GamePhase {
  DRAW = 'DRAW',
  MAIN_1 = 'MAIN_1',
  COMBAT = 'COMBAT',
  MAIN_2 = 'MAIN_2',
  END = 'END'
}
```

## 主要操作

```typescript
// 初始化对战
initializeMatch(matchData: MatchInitData): PayloadAction<MatchInitData>

// 加载对战数据
loadMatch(matchId: string): ThunkAction

// 更新对战状态
updateMatchState(matchState: Partial<CurrentMatchState>): PayloadAction<Partial<CurrentMatchState>>

// 更新玩家状态
updatePlayerState(playerId: string, update: Partial<PlayerState>): PayloadAction<{playerId: string, update: Partial<PlayerState>}>

// 更新游戏板状态
updateBoardState(update: Partial<BoardState>): PayloadAction<Partial<BoardState>>

// 设置当前回合玩家
setActivePlayer(playerId: string): PayloadAction<string>

// 更新游戏阶段
setGamePhase(phase: GamePhase): PayloadAction<GamePhase>

// 处理对战结束
endMatch(winner: string | null): PayloadAction<string | null>

// 设置对战错误
setMatchError(error: string | null): PayloadAction<string | null>

// 重置对战状态
resetMatchState(): PayloadAction
```

## 状态选择器

```typescript
// 获取当前对战ID
selectMatchId(state: RootState): string | null

// 获取当前玩家ID
selectCurrentPlayerId(state: RootState): string | null

// 获取对手玩家ID
selectOpponentId(state: RootState): string | null

// 检查是否为玩家的回合
selectIsPlayerTurn(state: RootState): boolean

// 获取当前游戏阶段
selectCurrentPhase(state: RootState): GamePhase | null

// 获取当前玩家的生命值
selectPlayerHealth(state: RootState): number

// 获取对手玩家的生命值
selectOpponentHealth(state: RootState): number

// 获取当前玩家的手牌
selectPlayerHand(state: RootState): Card[]

// 获取玩家场上的卡牌
selectPlayerField(state: RootState): Card[]

// 获取对手场上的卡牌
selectOpponentField(state: RootState): Card[]

// 检查对战是否已结束
selectIsMatchEnded(state: RootState): boolean

// 获取对战胜者
selectMatchWinner(state: RootState): string | null
```

## 自定义钩子

```typescript
// 使用当前对战状态
useCurrentMatch(): {
  matchId: string | null;
  gameMode: GameMode | null;
  isPlayerTurn: boolean;
  currentPhase: GamePhase | null;
  isMatchEnded: boolean;
  // ...其他状态
}

// 使用玩家状态
usePlayerState(playerId?: string): PlayerState | null

// 使用游戏板状态
useBoardState(): BoardState

// 处理回合变化
useTurnChange(callback: (playerId: string) => void): void

// 处理阶段变化
usePhaseChange(callback: (phase: GamePhase) => void): void

// 对战结束监听
useMatchEnd(callback: (winner: string | null) => void): void
```

## 与其他子模块的关系

- 为**card-play**子模块提供核心状态管理
- 为**card-draw**子模块提供牌库和手牌状态
- 为**in-game-interactions**子模块提供交互状态基础
- 为**match-interactions**子模块提供回合和阶段状态
- 为**match-results**子模块提供结果判定状态
- 为**ui**子目录提供展示所需的状态数据

## 开发指南

1. 状态定义应遵循单一数据源原则，避免状态重复和不一致
2. 操作设计应满足最小权限原则，每个操作只修改必要的状态部分
3. 选择器应优化性能，避免不必要的重新计算和渲染
4. 状态更新应考虑服务器同步和冲突解决策略
5. 复杂状态计算应使用记忆化选择器，提高性能
6. 状态设计应考虑扩展性，便于添加新的游戏机制和功能
7. 钩子设计应关注组件使用便利性，抽象复杂状态逻辑
8. 错误处理应完善，确保状态操作的健壮性
9. 状态结构应有良好的文档和类型定义，便于团队协作 