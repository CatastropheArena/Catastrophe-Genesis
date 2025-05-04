# 大厅模型子目录 (Lobby Model)

## 概述

大厅模型子目录包含当前大厅模块的核心状态管理逻辑和数据结构。它定义了整个大厅会话中的状态管理架构，包括状态存储、操作定义、选择器和相关钩子。作为整个大厅功能的基础，模型子目录为所有大厅相关功能提供了一致的状态管理基础，确保大厅状态的一致性、可预测性和可维护性。

## 文件结构

```
model/
├── index.ts        # 模型导出
├── store.ts        # Redux状态存储定义
├── actions.ts      # 大厅核心操作
├── selectors.ts    # 状态选择器
└── hooks.ts        # 自定义React钩子
```

## 功能职责

### 核心状态定义

定义和管理大厅的核心状态数据：
- 大厅基本信息（ID、名称、创建者等）
- 玩家列表和状态（准备状态、队伍分配等）
- 大厅设置和规则参数
- 大厅生命周期状态（创建中、活跃、游戏开始等）
- 当前用户在大厅中的状态和权限

### 状态更新操作

提供修改大厅状态的操作集合：
- 初始化和加载大厅数据
- 更新玩家状态（准备/未准备）
- 修改大厅设置
- 处理玩家加入/离开
- 管理游戏启动流程

### 状态选择器

提供访问大厅状态的标准方式：
- 获取当前大厅信息
- 检索玩家列表和状态
- 查询大厅设置
- 判断当前用户权限
- 检查大厅状态条件（如是否可以开始游戏）

### 状态管理钩子

提供React组件使用的自定义钩子：
- 大厅状态读取钩子
- 大厅操作执行钩子
- 状态变化监听钩子
- 权限检查钩子
- 设置修改钩子

## 数据模型

大厅模型子目录的核心数据结构：

```typescript
// 大厅状态
export interface CurrentLobbyState {
  lobbyId: string | null;
  lobbyName: string | null;
  hostId: string | null;
  players: LobbyPlayer[];
  settings: LobbySettings;
  isLoading: boolean;
  isHost: boolean;
  isReady: boolean;
  error: string | null;
  startCountdown: number | null;
  createdAt: number | null;
}

// 大厅玩家
export interface LobbyPlayer {
  id: string;
  username: string;
  avatar: string;
  isHost: boolean;
  isReady: boolean;
  team: number;
  joinedAt: number;
}

// 大厅设置
export interface LobbySettings {
  gameMode: GameMode;
  maxPlayers: number;
  isPrivate: boolean;
  allowSpectators: boolean;
  timeLimit: number;
  customRules: CustomRules;
  map: string;
  teamSize: number;
}

// 游戏模式枚举
export enum GameMode {
  CASUAL = 'CASUAL',
  RANKED = 'RANKED',
  CUSTOM = 'CUSTOM',
  TOURNAMENT = 'TOURNAMENT'
}
```

## 主要操作

```typescript
// 初始化大厅
initializeLobby(lobbyData: InitialLobbyData): PayloadAction<InitialLobbyData>

// 加载大厅数据
loadLobby(lobbyId: string): ThunkAction

// 更新大厅状态
updateLobbyState(update: Partial<CurrentLobbyState>): PayloadAction<Partial<CurrentLobbyState>>

// 更新玩家列表
updatePlayers(players: LobbyPlayer[]): PayloadAction<LobbyPlayer[]>

// 更新单个玩家状态
updatePlayerState(playerId: string, update: Partial<LobbyPlayer>): PayloadAction<{playerId: string, update: Partial<LobbyPlayer>}>

// 更新大厅设置
updateSettings(settings: Partial<LobbySettings>): PayloadAction<Partial<LobbySettings>>

// 设置当前用户准备状态
setReady(isReady: boolean): ThunkAction

// 开始游戏倒计时
startCountdown(seconds: number): PayloadAction<number>

// 取消游戏倒计时
cancelCountdown(): PayloadAction

// 设置大厅错误
setLobbyError(error: string | null): PayloadAction<string | null>

// 重置大厅状态
resetLobbyState(): PayloadAction
```

## 状态选择器

```typescript
// 获取大厅ID
selectLobbyId(state: RootState): string | null

// 获取大厅名称
selectLobbyName(state: RootState): string | null

// 获取大厅玩家列表
selectPlayers(state: RootState): LobbyPlayer[]

// 获取主机ID
selectHostId(state: RootState): string | null

// 判断当前用户是否为主机
selectIsHost(state: RootState): boolean

// 获取当前用户准备状态
selectIsReady(state: RootState): boolean

// 获取大厅设置
selectSettings(state: RootState): LobbySettings

// 检查是否所有非主机玩家都已准备
selectAreAllPlayersReady(state: RootState): boolean

// 判断是否可以开始游戏
selectCanStartGame(state: RootState): boolean

// 获取倒计时状态
selectCountdown(state: RootState): number | null

// 检查大厅是否正在加载
selectIsLobbyLoading(state: RootState): boolean

// 获取大厅错误信息
selectLobbyError(state: RootState): string | null
```

## 自定义钩子

```typescript
// 使用当前大厅状态
useCurrentLobby(): {
  lobbyId: string | null;
  lobbyName: string | null;
  players: LobbyPlayer[];
  settings: LobbySettings;
  isHost: boolean;
  isReady: boolean;
  // ...其他状态
}

// 使用大厅操作
useLobbyActions(): {
  setReady: (isReady: boolean) => void;
  updateSettings: (settings: Partial<LobbySettings>) => void;
  kickPlayer: (playerId: string) => void;
  startGame: () => void;
  leaveLobby: () => void;
}

// 监听玩家变化
usePlayersChange(callback: (players: LobbyPlayer[]) => void): void

// 监听准备状态变化
useReadyStateChange(callback: (isReady: boolean) => void): void

// 监听倒计时变化
useCountdownChange(callback: (countdown: number | null) => void): void
```

## 与其他子模块的关系

- 为**lobby-settings**子模块提供设置状态管理
- 为**lobby-interactions**子模块提供玩家交互状态
- 向父级**current-lobby**模块提供核心状态和操作
- 与**chat**模块交互，提供大厅聊天上下文
- 向**current-match**模块传递游戏启动数据

## 开发指南

1. 状态定义应遵循单一数据源原则，避免状态重复和不一致
2. 操作设计应满足最小权限原则，每个操作只修改必要的状态部分
3. 选择器应优化性能，避免不必要的重新计算和重渲染
4. 状态更新应考虑网络同步和冲突解决策略
5. 复杂状态计算应使用记忆化选择器，提高性能
6. 状态设计应考虑扩展性，便于添加新的大厅功能
7. 钩子设计应关注组件使用便利性，抽象复杂状态逻辑
8. 错误处理应完善，确保状态操作的健壮性
9. 权限控制应严格，防止未授权操作 