# 对战互动子模块 (Match Interactions Feature)

## 概述

对战互动子模块是当前对战模块的关键组成部分，专注于处理玩家在对战基本流程中的各种交互和操作。该子模块管理回合流转、阶段切换、时间限制等核心对战机制，以及玩家在对战中可以执行的基础操作，如结束回合、认输、请求暂停等。作为对战流程控制的核心，对战互动子模块确保游戏规则正确执行，并为玩家提供直观的对战控制界面。

## 文件结构

```
match-interactions/
├── index.ts            # 模块导出
├── model/              # 对战互动状态管理
│   ├── index.ts        # 模型导出
│   ├── store.ts        # 状态管理
│   ├── actions.ts      # 互动操作
│   ├── selectors.ts    # 状态选择器
│   └── types.ts        # 类型定义
└── ui/                 # 对战互动界面组件
    ├── index.ts        # UI组件导出
    ├── turn-controls.tsx      # 回合控制组件
    ├── phase-indicator.tsx    # 阶段指示器
    ├── timer.tsx              # 时间限制计时器
    ├── surrender-dialog.tsx   # 认输对话框
    └── pause-request.tsx      # 暂停请求组件
```

## 功能职责

### 回合管理

处理游戏回合相关的操作和状态：
- 回合开始和结束控制
- 当前激活玩家追踪
- 回合计数和历史记录
- 回合超时处理
- 特殊回合规则（如额外回合）

### 阶段管理

控制和处理游戏阶段：
- 阶段切换和流转
- 阶段限时管理
- 阶段特定规则执行
- 可选阶段跳过
- 阶段触发事件处理

### 时间控制系统

管理对战中的各种时间限制：
- 回合计时器实现
- 玩家思考时间管理
- 时间储备机制
- 超时警告和处理
- 时间显示格式化

### 杂项对战操作

提供各种对战控制功能：
- 认输操作处理
- 暂停请求机制
- 撤销操作（如规则允许）
- 特殊行动（如表情、提示）
- 观战者视角控制

## 数据模型

对战互动子模块的核心数据结构：

```typescript
// 对战互动状态
export interface MatchInteractionsState {
  currentTurn: number;
  activePlayerId: string | null;
  currentPhase: GamePhase;
  phaseStartTime: number;
  turnStartTime: number;
  isPlayerTurn: boolean;
  turnTimeRemaining: number;
  phaseTimeRemaining: number;
  timeBank: Record<string, number>;
  autoPassPriority: boolean;
  availableActions: MatchAction[];
  pauseRequested: boolean;
  pauseRequestedBy: string | null;
  surrenderDialogOpen: boolean;
  turnHistory: TurnHistoryEntry[];
}

// 游戏阶段枚举
export enum GamePhase {
  DRAW = 'DRAW',
  MAIN_1 = 'MAIN_1',
  COMBAT = 'COMBAT',
  MAIN_2 = 'MAIN_2',
  END = 'END'
}

// 对战行动类型
export type MatchAction = 
  | 'END_TURN'
  | 'PASS_PRIORITY'
  | 'SURRENDER'
  | 'REQUEST_PAUSE'
  | 'CANCEL_PAUSE'
  | 'UNDO'
  | 'EMOTE';

// 回合历史记录
export interface TurnHistoryEntry {
  turnNumber: number;
  playerId: string;
  startTime: number;
  endTime: number | null;
  phases: PhaseHistoryEntry[];
  actions: ActionHistoryEntry[];
}

// 阶段历史记录
export interface PhaseHistoryEntry {
  phase: GamePhase;
  startTime: number;
  endTime: number | null;
  skipped: boolean;
}

// 行动历史记录
export interface ActionHistoryEntry {
  actionType: string;
  timestamp: number;
  playerId: string;
  metadata?: Record<string, any>;
}
```

## 主要操作

```typescript
// 结束当前回合
endTurn(): ThunkAction

// 进入下一阶段
nextPhase(): ThunkAction

// 跳过当前阶段
skipPhase(phase: GamePhase): ThunkAction

// 传递优先权
passPriority(): ThunkAction

// 请求暂停
requestPause(reason?: string): ThunkAction

// 取消暂停请求
cancelPauseRequest(): ThunkAction

// 执行认输
surrender(): ThunkAction

// 设置自动传递优先权
setAutoPassPriority(enabled: boolean): PayloadAction<boolean>

// 发送表情
sendEmote(emoteId: string): ThunkAction

// 更新计时器
updateTimer(): PayloadAction
```

## UI组件

### TurnControls

回合控制组件，提供结束回合、跳过阶段等功能。

**属性：**
- `isPlayerTurn: boolean` - 是否为玩家回合
- `currentPhase: GamePhase` - 当前阶段
- `availableActions: MatchAction[]` - 可用操作
- `onEndTurn: () => void` - 结束回合回调
- `onNextPhase: () => void` - 下一阶段回调
- `onPassPriority: () => void` - 传递优先权回调
- `disabled?: boolean` - 是否禁用

### PhaseIndicator

阶段指示器组件，显示当前游戏阶段和可能的下一阶段。

**属性：**
- `currentPhase: GamePhase` - 当前阶段
- `phases: GamePhase[]` - 所有阶段
- `isPlayerTurn: boolean` - 是否为玩家回合
- `activePlayerId: string` - 当前行动玩家ID
- `onPhaseClick?: (phase: GamePhase) => void` - 阶段点击回调
- `compact?: boolean` - 是否使用紧凑模式

### Timer

对战计时器组件，显示回合和阶段剩余时间。

**属性：**
- `timeRemaining: number` - 剩余时间（秒）
- `type: 'turn' | 'phase'` - 计时器类型
- `warningThreshold?: number` - 警告阈值（秒）
- `criticalThreshold?: number` - 危急阈值（秒）
- `format?: 'standard' | 'compact'` - 时间格式
- `showProgressBar?: boolean` - 是否显示进度条

### SurrenderDialog

认输对话框组件，确认玩家认输意图。

**属性：**
- `isOpen: boolean` - 是否显示
- `onConfirm: () => void` - 确认认输回调
- `onCancel: () => void` - 取消回调
- `matchData?: MatchDataSummary` - 对战摘要数据

### PauseRequest

暂停请求组件，显示和处理暂停请求。

**属性：**
- `isPauseRequested: boolean` - 是否请求暂停
- `requestedBy: string` - 请求者ID
- `requestReason?: string` - 暂停原因
- `onAccept: () => void` - 接受暂停回调
- `onDecline: () => void` - 拒绝暂停回调
- `timeToRespond?: number` - 响应时间限制（秒）

## 使用示例

### 回合控制集成

```tsx
import { useDispatch, useSelector } from 'react-redux';
import { 
  endTurn, 
  nextPhase,
  passPriority,
  surrender,
  selectIsPlayerTurn,
  selectCurrentPhase,
  selectAvailableActions,
  selectTurnTimeRemaining
} from '@features/current-match/match-interactions';
import { 
  TurnControls, 
  PhaseIndicator,
  Timer,
  SurrenderDialog
} from '@features/current-match/match-interactions/ui';
import { useState } from 'react';

const MatchControlPanel = () => {
  const dispatch = useDispatch();
  const isPlayerTurn = useSelector(selectIsPlayerTurn);
  const currentPhase = useSelector(selectCurrentPhase);
  const availableActions = useSelector(selectAvailableActions);
  const turnTimeRemaining = useSelector(selectTurnTimeRemaining);
  const [surrenderDialogOpen, setSurrenderDialogOpen] = useState(false);
  
  // 处理结束回合
  const handleEndTurn = () => {
    dispatch(endTurn());
  };
  
  // 处理下一阶段
  const handleNextPhase = () => {
    dispatch(nextPhase());
  };
  
  // 处理传递优先权
  const handlePassPriority = () => {
    dispatch(passPriority());
  };
  
  // 打开认输对话框
  const handleOpenSurrenderDialog = () => {
    setSurrenderDialogOpen(true);
  };
  
  // 确认认输
  const handleConfirmSurrender = () => {
    dispatch(surrender());
    setSurrenderDialogOpen(false);
  };
  
  return (
    <div className="match-control-panel">
      {/* 回合计时器 */}
      <Timer 
        timeRemaining={turnTimeRemaining}
        type="turn"
        warningThreshold={30}
        criticalThreshold={10}
        showProgressBar={true}
      />
      
      {/* 阶段指示器 */}
      <PhaseIndicator 
        currentPhase={currentPhase}
        phases={[
          'DRAW',
          'MAIN_1',
          'COMBAT',
          'MAIN_2',
          'END'
        ]}
        isPlayerTurn={isPlayerTurn}
        activePlayerId={currentPlayerId}
      />
      
      {/* 回合控制 */}
      <TurnControls 
        isPlayerTurn={isPlayerTurn}
        currentPhase={currentPhase}
        availableActions={availableActions}
        onEndTurn={handleEndTurn}
        onNextPhase={handleNextPhase}
        onPassPriority={handlePassPriority}
        disabled={isAnimationPlaying}
      />
      
      {/* 额外操作按钮 */}
      <div className="additional-controls">
        <button 
          className="surrender-button"
          onClick={handleOpenSurrenderDialog}
        >
          认输
        </button>
      </div>
      
      {/* 认输确认对话框 */}
      <SurrenderDialog 
        isOpen={surrenderDialogOpen}
        onConfirm={handleConfirmSurrender}
        onCancel={() => setSurrenderDialogOpen(false)}
      />
    </div>
  );
};
```

### 暂停请求处理

```tsx
import { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { 
  requestPause, 
  cancelPauseRequest,
  selectIsPauseRequested,
  selectPauseRequestedBy
} from '@features/current-match/match-interactions';
import { 
  PauseRequest 
} from '@features/current-match/match-interactions/ui';
import { selectCurrentPlayerId } from '@features/current-match';

const PauseRequestHandler = () => {
  const dispatch = useDispatch();
  const isPauseRequested = useSelector(selectIsPauseRequested);
  const pauseRequestedBy = useSelector(selectPauseRequestedBy);
  const currentPlayerId = useSelector(selectCurrentPlayerId);
  
  // 清理暂停请求
  useEffect(() => {
    return () => {
      // 组件卸载时取消自己的暂停请求
      if (pauseRequestedBy === currentPlayerId) {
        dispatch(cancelPauseRequest());
      }
    };
  }, [dispatch, pauseRequestedBy, currentPlayerId]);
  
  // 请求暂停
  const handleRequestPause = (reason) => {
    dispatch(requestPause(reason));
  };
  
  // 取消暂停
  const handleCancelPause = () => {
    dispatch(cancelPauseRequest());
  };
  
  // 接受对方暂停请求
  const handleAcceptPause = () => {
    // 发送接受暂停的消息，服务器会暂停游戏
    dispatch(acceptPauseRequest());
  };
  
  // 拒绝对方暂停请求
  const handleDeclinePause = () => {
    // 发送拒绝暂停的消息
    dispatch(declinePauseRequest());
  };
  
  return (
    <>
      {/* 暂停请求按钮 */}
      {!isPauseRequested && (
        <button 
          className="pause-request-button"
          onClick={() => handleRequestPause('需要短暂休息')}
        >
          请求暂停
        </button>
      )}
      
      {/* 已请求暂停 */}
      {isPauseRequested && pauseRequestedBy === currentPlayerId && (
        <div className="pause-pending">
          <span>暂停请求已发送...</span>
          <button onClick={handleCancelPause}>
            取消请求
          </button>
        </div>
      )}
      
      {/* 收到对方暂停请求 */}
      {isPauseRequested && pauseRequestedBy !== currentPlayerId && (
        <PauseRequest 
          isPauseRequested={isPauseRequested}
          requestedBy={pauseRequestedBy}
          requestReason="需要短暂休息"
          onAccept={handleAcceptPause}
          onDecline={handleDeclinePause}
          timeToRespond={30}
        />
      )}
    </>
  );
};
```

## 与其他模块的关系

- 为**card-play**子模块提供回合和阶段的上下文，确定何时可以出牌
- 与**card-draw**子模块协作，处理回合开始时的抽牌
- 基于**in-game-interactions**子模块的交互框架
- 与**match-results**子模块交互，在认输时触发游戏结束
- 更新**current-match**模块的核心游戏状态
- 可能与游戏规则模块交互，确保回合和阶段切换符合规则

## 开发指南

1. 回合管理应严格遵循游戏规则，确保公平性和一致性
2. 时间控制是竞技游戏的关键，应提供精确且用户友好的计时机制
3. 对战互动UI应简明直观，避免玩家在关键时刻因界面复杂而犯错
4. 认输和暂停等特殊操作应有适当的确认机制，防止误触
5. 适当记录回合历史，便于回放和分析
6. 考虑网络延迟情况，提供状态同步机制
7. 实现回合超时的优雅处理，确保游戏流程不会因玩家离开而卡住
8. 提供适当的键盘快捷键，提升用户体验
9. 对战控制应考虑观战者视角，以不同方式展示信息 