# 大厅互动子模块 (Lobby Interactions Feature)

## 概述

大厅互动子模块是当前大厅功能的交互核心，专注于处理玩家在大厅中的各种互动行为和操作。该子模块负责管理玩家准备状态、主机控制功能、玩家管理、团队分配、大厅聊天整合以及游戏启动流程。它为玩家提供了一个社交化的游戏前准备环境，增强了玩家之间的互动体验和游戏前的组织协调能力。

## 文件结构

```
lobby-interactions/
├── index.ts            # 模块导出
├── model/              # 互动状态管理
│   ├── index.ts        # 模型导出
│   ├── actions.ts      # 互动操作
│   ├── selectors.ts    # 状态选择器
│   └── middleware.ts   # 互动中间件
└── ui/                 # 互动界面组件
    ├── index.ts        # UI组件导出
    ├── player-list.tsx         # 玩家列表组件
    ├── ready-button.tsx        # 准备按钮组件
    ├── host-controls.tsx       # 主机控制组件
    ├── team-selector.tsx       # 队伍选择器
    └── lobby-chat.tsx          # 大厅聊天组件
```

## 功能职责

### 玩家状态管理

管理玩家在大厅中的状态和操作：
- 准备/取消准备状态控制
- 玩家加入和离开处理
- 玩家信息展示和更新
- 在线状态和连接监控
- 玩家操作权限控制

### 主机控制功能

提供大厅主机特有的控制功能：
- 踢出和禁止玩家
- 转移主机权限
- 开始游戏控制
- 强制玩家就绪
- 锁定/解锁大厅

### 团队组织

管理多人游戏的团队分配：
- 队伍创建和管理
- 玩家分配到队伍
- 平衡团队功能
- 队伍限制和规则
- 队伍颜色和标识

### 社交互动

提供大厅内的社交功能：
- 大厅聊天集成
- 表情和快捷消息
- 玩家间互动操作
- 好友邀请功能
- 状态和活动广播

### 游戏启动流程

协调从大厅到游戏的过渡过程：
- 准备状态验证
- 游戏启动倒计时
- 玩家确认检查
- 最终设置应用
- 平滑过渡到游戏界面

## 数据模型

大厅互动子模块的核心数据结构：

```typescript
// 互动状态
export interface LobbyInteractionsState {
  playerActions: PlayerActions;
  teamAssignments: TeamAssignments;
  chatIntegration: ChatIntegration;
  gameStartProcess: GameStartProcess;
  contextMenuState: ContextMenuState | null;
}

// 玩家操作状态
export interface PlayerActions {
  readyPlayersIds: string[];
  kickingPlayerId: string | null;
  invitingFriendId: string | null;
  transferringHostTo: string | null;
  playerContextMenu: PlayerContextMenu | null;
}

// 团队分配状态
export interface TeamAssignments {
  teams: Record<number, Team>;
  playerTeams: Record<string, number>;
  autoBalance: boolean;
  allowTeamChange: boolean;
  lockedTeams: boolean;
}

// 团队信息
export interface Team {
  id: number;
  name: string;
  color: string;
  maxSize: number;
  playerIds: string[];
}

// 聊天集成
export interface ChatIntegration {
  chatContextId: string | null;
  unreadMessages: number;
  lastMessageTime: number | null;
  isMinimized: boolean;
}

// 游戏启动流程
export interface GameStartProcess {
  isStarting: boolean;
  countdown: number | null;
  readyConfirmation: Record<string, boolean>;
  countdownStartTime: number | null;
  abortedBy: string | null;
}

// 玩家上下文菜单
export interface PlayerContextMenu {
  playerId: string;
  position: { x: number, y: number };
  options: ContextMenuOption[];
}

// 菜单选项
export interface ContextMenuOption {
  id: string;
  label: string;
  icon?: string;
  action: () => void;
  disabled?: boolean;
}
```

## 主要操作

```typescript
// 准备状态控制
toggleReady(): ThunkAction

// 踢出玩家
kickPlayer(playerId: string): ThunkAction

// 转移主机
transferHost(playerId: string): ThunkAction

// 切换队伍
changeTeam(teamId: number): ThunkAction

// 分配玩家到队伍
assignPlayerToTeam(playerId: string, teamId: number): ThunkAction

// 平衡队伍
balanceTeams(): ThunkAction

// 开始游戏
startGame(): ThunkAction

// 取消游戏开始
cancelGameStart(): ThunkAction

// 确认准备就绪
confirmReady(): ThunkAction

// 打开玩家上下文菜单
openPlayerContextMenu(options: PlayerContextMenuOptions): PayloadAction<PlayerContextMenuOptions>

// 关闭上下文菜单
closeContextMenu(): PayloadAction

// 设置聊天集成
setChatContext(contextId: string): PayloadAction<string>

// 邀请好友
inviteFriend(friendId: string): ThunkAction

// 锁定大厅
lockLobby(locked: boolean): ThunkAction
```

## UI组件

### PlayerList

玩家列表组件，显示大厅中的所有玩家及其状态。

**属性：**
- `players: LobbyPlayer[]` - 玩家列表
- `currentUserId: string` - 当前用户ID
- `isHost: boolean` - 当前用户是否为主机
- `readyPlayers: string[]` - 已准备的玩家ID列表
- `onPlayerClick?: (playerId: string) => void` - 玩家点击回调
- `onPlayerContextMenu?: (playerId: string, event: React.MouseEvent) => void` - 玩家右键菜单回调
- `showDetails?: boolean` - 是否显示详细信息

### ReadyButton

准备状态切换按钮，用于玩家表示准备就绪。

**属性：**
- `isReady: boolean` - 当前准备状态
- `onChange: (isReady: boolean) => void` - 状态变更回调
- `isHost: boolean` - 是否为主机
- `disabled?: boolean` - 是否禁用
- `size?: 'small' | 'medium' | 'large'` - 按钮大小
- `withLabel?: boolean` - 是否显示文字标签

### HostControls

主机控制面板，提供主机特有的操作选项。

**属性：**
- `canStartGame: boolean` - 是否可以开始游戏
- `onStartGame: () => void` - 开始游戏回调
- `onKickPlayer: (playerId: string) => void` - 踢出玩家回调
- `onTransferHost: (playerId: string) => void` - 转移主机回调
- `onLockLobby: (locked: boolean) => void` - 锁定大厅回调
- `isLobbyLocked: boolean` - 大厅是否已锁定
- `compact?: boolean` - 紧凑模式

### TeamSelector

团队选择器组件，用于查看和管理游戏团队。

**属性：**
- `teams: Team[]` - 队伍列表
- `playerTeams: Record<string, number>` - 玩家所属队伍映射
- `currentUserId: string` - 当前用户ID
- `onTeamChange: (teamId: number) => void` - 更换队伍回调
- `onPlayerAssign: (playerId: string, teamId: number) => void` - 分配玩家回调
- `canChangeTeam: boolean` - 是否可以更换队伍
- `isHost: boolean` - 是否为主机
- `onBalanceTeams?: () => void` - 平衡队伍回调

### LobbyChat

大厅聊天组件，集成聊天功能到大厅界面。

**属性：**
- `contextId: string` - 聊天上下文ID
- `minimized: boolean` - 是否最小化
- `onToggleMinimize: () => void` - 切换最小化回调
- `unreadCount: number` - 未读消息数量
- `maxHeight?: string` - 最大高度
- `width?: string` - 宽度
- `position?: 'left' | 'right' | 'bottom'` - 位置

## 使用示例

### 玩家管理与准备状态

```tsx
import { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { 
  PlayerList,
  ReadyButton,
  toggleReady,
  kickPlayer,
  openPlayerContextMenu,
  selectReadyPlayersIds
} from '@features/current-lobby/lobby-interactions';
import { 
  selectPlayers, 
  selectIsHost, 
  selectIsReady,
  selectCurrentUserId
} from '@features/current-lobby';

const LobbyPlayersPanel = () => {
  const dispatch = useDispatch();
  const players = useSelector(selectPlayers);
  const isHost = useSelector(selectIsHost);
  const isReady = useSelector(selectIsReady);
  const currentUserId = useSelector(selectCurrentUserId);
  const readyPlayersIds = useSelector(selectReadyPlayersIds);
  
  // 处理准备状态切换
  const handleReadyChange = (ready) => {
    dispatch(toggleReady());
  };
  
  // 处理玩家右键菜单
  const handlePlayerContextMenu = (playerId, event) => {
    event.preventDefault();
    
    // 只有主机可以看到踢出和转移主机选项
    const menuOptions = [
      {
        id: 'view-profile',
        label: '查看资料',
        icon: 'user',
        action: () => viewPlayerProfile(playerId)
      },
      {
        id: 'add-friend',
        label: '添加好友',
        icon: 'user-plus',
        action: () => addFriend(playerId),
        disabled: isFriend(playerId)
      }
    ];
    
    // 主机特权选项
    if (isHost && playerId !== currentUserId) {
      menuOptions.push(
        {
          id: 'kick',
          label: '踢出玩家',
          icon: 'user-x',
          action: () => dispatch(kickPlayer(playerId))
        },
        {
          id: 'transfer-host',
          label: '转移主机',
          icon: 'crown',
          action: () => dispatch(transferHost(playerId))
        }
      );
    }
    
    dispatch(openPlayerContextMenu({
      playerId,
      position: { x: event.clientX, y: event.clientY },
      options: menuOptions
    }));
  };
  
  return (
    <div className="lobby-players-panel">
      <div className="panel-header">
        <h3>玩家列表</h3>
        <div className="player-count">
          {players.length} / {maxPlayers}
        </div>
      </div>
      
      <PlayerList 
        players={players}
        currentUserId={currentUserId}
        isHost={isHost}
        readyPlayers={readyPlayersIds}
        onPlayerContextMenu={handlePlayerContextMenu}
        showDetails={true}
      />
      
      <div className="ready-control">
        <ReadyButton 
          isReady={isReady}
          onChange={handleReadyChange}
          isHost={isHost}
          withLabel={true}
          size="large"
        />
      </div>
    </div>
  );
};
```

### 主机控制面板

```tsx
import { useDispatch, useSelector } from 'react-redux';
import { 
  HostControls,
  startGame,
  lockLobby,
  balanceTeams,
  selectGameStartProcess,
  selectIsLobbyLocked
} from '@features/current-lobby/lobby-interactions';
import { 
  selectIsHost, 
  selectCanStartGame
} from '@features/current-lobby';

const LobbyHostPanel = () => {
  const dispatch = useDispatch();
  const isHost = useSelector(selectIsHost);
  const canStartGame = useSelector(selectCanStartGame);
  const gameStartProcess = useSelector(selectGameStartProcess);
  const isLobbyLocked = useSelector(selectIsLobbyLocked);
  
  // 如果不是主机，不显示此面板
  if (!isHost) return null;
  
  // 处理开始游戏
  const handleStartGame = () => {
    dispatch(startGame());
  };
  
  // 处理锁定大厅
  const handleLockLobby = (locked) => {
    dispatch(lockLobby(locked));
  };
  
  // 处理平衡队伍
  const handleBalanceTeams = () => {
    dispatch(balanceTeams());
  };
  
  return (
    <div className="lobby-host-panel">
      <h3>主机控制</h3>
      
      <HostControls 
        canStartGame={canStartGame}
        onStartGame={handleStartGame}
        onLockLobby={handleLockLobby}
        isLobbyLocked={isLobbyLocked}
        onKickPlayer={(playerId) => dispatch(kickPlayer(playerId))}
        onTransferHost={(playerId) => dispatch(transferHost(playerId))}
      />
      
      {gameStartProcess.isStarting && (
        <div className="game-starting">
          <span>游戏即将开始，倒计时: {gameStartProcess.countdown}</span>
          <button onClick={() => dispatch(cancelGameStart())}>
            取消
          </button>
        </div>
      )}
      
      {/* 队伍平衡按钮（如果是团队游戏） */}
      {isTeamMode && (
        <button 
          className="balance-teams-button"
          onClick={handleBalanceTeams}
        >
          平衡队伍
        </button>
      )}
    </div>
  );
};
```

### 团队管理与聊天集成

```tsx
import { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { 
  TeamSelector,
  LobbyChat,
  changeTeam,
  assignPlayerToTeam,
  setChatContext,
  selectTeams,
  selectPlayerTeams,
  selectChatIntegration
} from '@features/current-lobby/lobby-interactions';
import { 
  selectCurrentUserId, 
  selectIsHost,
  selectSettings
} from '@features/current-lobby';

const LobbySocialPanel = () => {
  const dispatch = useDispatch();
  const currentUserId = useSelector(selectCurrentUserId);
  const isHost = useSelector(selectIsHost);
  const settings = useSelector(selectSettings);
  const teams = useSelector(selectTeams);
  const playerTeams = useSelector(selectPlayerTeams);
  const chatIntegration = useSelector(selectChatIntegration);
  
  // 设置聊天上下文ID
  useEffect(() => {
    if (lobbyId) {
      const chatContextId = `lobby-${lobbyId}`;
      dispatch(setChatContext(chatContextId));
    }
  }, [lobbyId, dispatch]);
  
  // 处理队伍变更
  const handleTeamChange = (teamId) => {
    dispatch(changeTeam(teamId));
  };
  
  // 处理玩家分配（主机功能）
  const handlePlayerAssign = (playerId, teamId) => {
    dispatch(assignPlayerToTeam(playerId, teamId));
  };
  
  // 切换聊天最小化状态
  const toggleChatMinimize = () => {
    dispatch({
      type: 'lobbyInteractions/toggleChat',
      payload: !chatIntegration.isMinimized
    });
  };
  
  return (
    <div className="lobby-social-panel">
      {/* 团队选择器（如果是团队游戏） */}
      {settings.teamSize > 1 && (
        <div className="teams-section">
          <h3>队伍</h3>
          <TeamSelector 
            teams={Object.values(teams)}
            playerTeams={playerTeams}
            currentUserId={currentUserId}
            onTeamChange={handleTeamChange}
            onPlayerAssign={handlePlayerAssign}
            canChangeTeam={settings.allowTeamChange}
            isHost={isHost}
          />
        </div>
      )}
      
      {/* 大厅聊天 */}
      <LobbyChat 
        contextId={chatIntegration.chatContextId}
        minimized={chatIntegration.isMinimized}
        onToggleMinimize={toggleChatMinimize}
        unreadCount={chatIntegration.unreadMessages}
        position="right"
      />
    </div>
  );
};
```

## 与其他子模块的关系

- 使用**model**子模块的核心状态和操作
- 与**lobby-settings**子模块协作，基于设置提供适当的互动选项
- 整合**chat**模块的功能，提供大厅内聊天
- 向**current-match**模块传递玩家团队和准备状态
- 与**lobby-rejoin**模块交互，处理玩家断线后重新加入的状态恢复
- 向父级**current-lobby**模块提供玩家互动功能

## 开发指南

1. 交互设计应直观且一致，提供清晰的视觉反馈
2. 主机和普通玩家的权限边界应明确，UI应据此调整
3. 团队管理功能应考虑平衡性，避免单方面压倒优势
4. 聊天集成应无缝，但不应干扰主要大厅功能
5. 游戏启动流程应可靠且有足够的确认机制
6. 社交功能应促进玩家间积极互动，同时提供适当的管控
7. 上下文菜单和快捷操作应便于访问，提高用户体验
8. 状态变化应实时反映给所有玩家，保持大厅状态同步
9. 错误和异常情况（如玩家突然离开）应优雅处理 