# 对战界面子目录 (Match UI)

## 概述

对战界面子目录包含当前对战模块的所有用户界面组件，负责呈现对战状态、处理用户交互，以及提供视觉反馈。这些组件构成了玩家在游戏对战过程中看到和交互的核心界面元素，包括游戏板、玩家信息、卡牌展示、状态指示器等。作为用户与游戏系统之间的桥梁，UI组件确保玩家能够清晰地了解游戏状态，并以直观的方式进行交互。

## 文件结构

```
ui/
├── index.ts                     # UI组件导出
├── players.tsx                  # 玩家信息展示组件
├── state-timer.tsx              # 状态计时器组件
├── context-indicator.tsx        # 上下文指示器组件
├── discard-pile.tsx             # 弃牌堆组件
└── exploding-kitten-probability.tsx  # 爆炸猫概率计算组件
```

## 核心组件

### players.tsx

玩家信息展示组件，显示对战双方的基本信息和状态。

**功能职责**:
- 显示玩家头像、名称和等级
- 呈现玩家当前生命值和法力值
- 展示玩家特殊状态和效果
- 提供玩家交互功能（如查看详情）
- 突出显示当前行动玩家

**主要组件**:
- `PlayerInfo`: 展示单个玩家信息的组件
- `OpponentInfo`: 特化的对手信息展示组件
- `PlayerStatus`: 显示玩家状态效果的组件
- `ResourceBar`: 可视化资源(生命值/法力值)的组件

### state-timer.tsx

状态计时器组件，用于显示和管理对战中的各种时间限制。

**功能职责**:
- 显示回合时间倒计时
- 提供阶段时间限制指示
- 在时间不足时提供警告
- 支持动态时间调整
- 处理计时器暂停和恢复

**主要组件**:
- `TurnTimer`: 回合计时器组件
- `PhaseTimer`: 阶段计时器组件
- `TimeBank`: 玩家时间储备显示组件
- `TimerWarning`: 时间不足警告组件

### context-indicator.tsx

上下文指示器组件，向玩家提供当前游戏状态的视觉提示。

**功能职责**:
- 显示当前游戏阶段
- 指示有效操作区域
- 提供行动提示信息
- 突出显示重要游戏事件
- 传达游戏规则上下文

**主要组件**:
- `PhaseIndicator`: 游戏阶段指示组件
- `ActionPrompt`: 玩家行动提示组件
- `EffectIndicator`: 正在生效的效果指示器
- `ContextTooltip`: 上下文信息提示组件

### discard-pile.tsx

弃牌堆组件，用于管理和显示已经使用或丢弃的卡牌。

**功能职责**:
- 显示弃牌堆中的卡牌数量
- 提供查看弃牌堆内容的功能
- 支持卡牌进入弃牌堆的动画
- 处理与弃牌堆相关的交互
- 突出显示重要弃牌

**主要组件**:
- `DiscardPile`: 主弃牌堆组件
- `DiscardViewer`: 弃牌堆内容查看器
- `DiscardAnimation`: 卡牌进入弃牌堆动画
- `ImportantDiscardIndicator`: 重要弃牌标识

### exploding-kitten-probability.tsx

爆炸猫概率计算组件，显示抽到特定卡牌的概率（特定游戏机制相关）。

**功能职责**:
- 计算并显示抽到特定卡牌的概率
- 根据牌库和已知信息更新概率
- 提供概率变化的可视化展示
- 在关键阈值提供风险警告
- 支持不同难度级别的显示方式

**主要组件**:
- `ProbabilityDisplay`: 概率数值显示组件
- `ProbabilityGauge`: 概率可视化仪表盘
- `RiskIndicator`: 风险等级指示器
- `ProbabilityTrend`: 概率变化趋势图

## 设计原则

界面组件的设计遵循以下原则：

1. **清晰性** - 游戏状态应当清晰易懂，避免信息过载
2. **一致性** - 保持UI风格和交互方式的一致性
3. **响应性** - 对用户操作提供即时反馈
4. **可扩展性** - 组件设计应便于扩展和定制
5. **性能优化** - 避免不必要的重渲染和计算
6. **可访问性** - 考虑不同用户的需求和偏好
7. **视觉层次** - 建立清晰的视觉层次结构，突出重要信息
8. **动态反馈** - 通过动画和过渡提供状态变化的视觉反馈

## 使用示例

### 玩家信息展示

```tsx
import { useSelector } from 'react-redux';
import { PlayerInfo } from '@features/current-match/ui';
import { 
  selectCurrentPlayerId, 
  selectOpponentId, 
  selectPlayerState 
} from '@features/current-match/model';

const PlayersDisplay = () => {
  const currentPlayerId = useSelector(selectCurrentPlayerId);
  const opponentId = useSelector(selectOpponentId);
  
  const playerState = useSelector(state => 
    selectPlayerState(state, currentPlayerId)
  );
  
  const opponentState = useSelector(state => 
    selectPlayerState(state, opponentId)
  );
  
  return (
    <div className="players-container">
      {/* 对手信息 */}
      <PlayerInfo 
        player={opponentState}
        isOpponent={true}
        showDetailedStats={false}
        animateResourceChanges={true}
      />
      
      {/* 玩家信息 */}
      <PlayerInfo 
        player={playerState}
        isOpponent={false}
        showDetailedStats={true}
        animateResourceChanges={true}
      />
    </div>
  );
};
```

### 回合计时器集成

```tsx
import { useSelector } from 'react-redux';
import { TurnTimer } from '@features/current-match/ui';
import { 
  selectIsPlayerTurn, 
  selectTurnTimeRemaining,
  selectPhase
} from '@features/current-match/model';

const GameTimerDisplay = () => {
  const isPlayerTurn = useSelector(selectIsPlayerTurn);
  const timeRemaining = useSelector(selectTurnTimeRemaining);
  const currentPhase = useSelector(selectPhase);
  
  return (
    <div className="game-timer">
      <TurnTimer 
        timeRemaining={timeRemaining}
        isPlayerTurn={isPlayerTurn}
        currentPhase={currentPhase}
        warningThreshold={30}
        criticalThreshold={10}
        showPulseAnimation={true}
      />
    </div>
  );
};
```

### 上下文指示器使用

```tsx
import { useSelector } from 'react-redux';
import { ContextIndicator } from '@features/current-match/ui';
import { 
  selectCurrentPhase,
  selectIsPlayerTurn,
  selectAvailableActions
} from '@features/current-match/model';

const GameContextDisplay = () => {
  const currentPhase = useSelector(selectCurrentPhase);
  const isPlayerTurn = useSelector(selectIsPlayerTurn);
  const availableActions = useSelector(selectAvailableActions);
  
  return (
    <div className="game-context">
      <ContextIndicator 
        phase={currentPhase}
        isPlayerTurn={isPlayerTurn}
        availableActions={availableActions}
        showHints={true}
        compact={false}
      />
    </div>
  );
};
```

### 弃牌堆组件使用

```tsx
import { useState } from 'react';
import { useSelector } from 'react-redux';
import { DiscardPile } from '@features/current-match/ui';
import { 
  selectPlayerGraveyard,
  selectOpponentGraveyard
} from '@features/current-match/model';

const GraveyardDisplay = () => {
  const [isViewerOpen, setIsViewerOpen] = useState(false);
  const playerGraveyard = useSelector(selectPlayerGraveyard);
  const opponentGraveyard = useSelector(selectOpponentGraveyard);
  
  const handlePileClick = () => {
    setIsViewerOpen(true);
  };
  
  return (
    <div className="graveyard-display">
      <DiscardPile 
        cards={playerGraveyard}
        label="你的弃牌堆"
        onClick={handlePileClick}
        showCount={true}
        highlightImportant={true}
        isOpen={isViewerOpen}
        onClose={() => setIsViewerOpen(false)}
      />
      
      <DiscardPile 
        cards={opponentGraveyard}
        label="对手的弃牌堆"
        onClick={handlePileClick}
        showCount={true}
        highlightImportant={true}
      />
    </div>
  );
};
```

## 主题与样式

界面组件支持主题定制，通过以下方式：

1. **主题变量** - 使用CSS变量定义主题颜色和尺寸
2. **样式变体** - 支持不同的视觉样式变体
3. **响应式设计** - 自适应不同屏幕尺寸
4. **动画定制** - 可配置的动画和过渡效果
5. **无障碍选项** - 支持高对比度和其他无障碍设置

## 与其他子模块的关系

- 使用**model**子目录提供的状态和选择器
- 与**card-play**子模块协作，展示卡牌出牌界面
- 与**card-draw**子模块交互，显示抽牌相关界面
- 与**in-game-interactions**子模块集成，处理交互反馈
- 与**match-interactions**子模块协作，提供回合控制界面
- 与**match-results**子模块配合，展示阶段性结果

## 开发指南

1. 组件设计应遵循原子设计原则，从基础组件构建复杂界面
2. 优先使用函数组件和React Hooks，便于状态管理和逻辑复用
3. 复杂组件应分解为更小的组件，提高可维护性
4. 使用React.memo和useCallback优化性能，避免不必要的重渲染
5. 动画应使用CSS动画或React专用动画库，保证性能
6. 所有组件应支持适当的属性验证和默认值
7. 交互状态应有明确的视觉反馈，如悬停、点击和禁用状态
8. 考虑极端情况，如长文本、空状态和错误状态的处理
9. 组件文档应详细，包括使用示例和各种配置选项 