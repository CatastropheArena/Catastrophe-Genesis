# 抽牌功能子模块 (Card Draw Feature)

## 概述

抽牌功能子模块是当前对战模块的重要组成部分，专注于管理游戏中的抽牌机制和相关状态。该子模块负责处理玩家从牌库抽取卡牌的完整流程，包括常规抽牌、特殊抽牌效果、抽牌动画、牌库状态追踪以及牌库耗尽的处理。作为资源管理的关键环节，抽牌功能直接影响游戏的节奏和策略深度。

## 文件结构

```
card-draw/
├── index.ts            # 模块导出
├── model/              # 抽牌状态管理
│   ├── index.ts        # 模型导出
│   ├── actions.ts      # 抽牌相关操作
│   ├── selectors.ts    # 状态选择器
│   ├── types.ts        # 类型定义
│   └── reducer.ts      # 抽牌状态reducer
└── ui/                 # 抽牌界面组件
    ├── index.ts        # UI组件导出
    ├── draw-animation.tsx       # 抽牌动画组件
    ├── deck-counter.tsx         # 牌库计数器
    └── draw-effect.tsx          # 抽牌特效组件
```

## 功能职责

### 抽牌操作管理

处理游戏中的各种抽牌操作：
- 回合开始时的常规抽牌
- 卡牌效果触发的额外抽牌
- 强制抽牌（如对手使用效果）
- 特殊抽牌（如查看并选择）
- 多张抽牌的顺序处理

### 牌库状态追踪

监控和管理牌库的状态：
- 追踪牌库剩余卡牌数量
- 处理牌库耗尽情况
- 管理牌库顶部可见卡牌
- 追踪已抽取的卡牌历史
- 计算抽牌概率（特定卡牌）

### 抽牌动画与视觉反馈

提供流畅的抽牌视觉体验：
- 从牌库到手牌的卡牌移动动画
- 抽牌时的特效展示
- 多张抽牌的序列动画
- 牌库变化的视觉更新
- 特殊抽牌的独特视觉效果

### 抽牌结果处理

管理抽牌后的状态更新：
- 更新手牌和牌库状态
- 处理手牌上限溢出
- 触发"抽牌时"效果
- 检查牌库耗尽导致的游戏结束条件
- 通知其他系统抽牌事件

## 数据模型

抽牌功能子模块的核心数据结构：

```typescript
// 抽牌状态
export interface CardDrawState {
  isDrawing: boolean;
  queuedDraws: DrawOperation[];
  lastDrawnCards: DrawnCardInfo[];
  deckSizes: Record<string, number>;
  topDeckVisible: Record<string, boolean>;
  topDeckCards: Record<string, Card | null>;
  drawHistory: DrawHistoryEntry[];
  error: string | null;
}

// 抽牌操作
export interface DrawOperation {
  playerId: string;
  count: number;
  source: DrawSource;
  animation: boolean;
  reveal: boolean;
  id: string;
}

// 抽牌来源枚举
export enum DrawSource {
  TURN_START = 'TURN_START',
  CARD_EFFECT = 'CARD_EFFECT',
  ABILITY = 'ABILITY',
  SPECIAL = 'SPECIAL'
}

// 已抽取卡牌信息
export interface DrawnCardInfo {
  cardId: string;
  playerId: string;
  timestamp: number;
  source: DrawSource;
  index: number;
  revealed: boolean;
}

// 抽牌历史记录
export interface DrawHistoryEntry {
  timestamp: number;
  playerId: string;
  count: number;
  source: DrawSource;
  cardIds: string[];
}
```

## 主要操作

```typescript
// 执行抽牌操作
drawCards(options: DrawCardsOptions): ThunkAction

// 处理抽牌响应
handleDrawResponse(response: DrawResponse): PayloadAction<DrawResponse>

// 查看牌库顶牌
viewTopDeck(playerId: string): ThunkAction

// 添加抽牌到队列
queueDraw(drawOp: DrawOperation): PayloadAction<DrawOperation>

// 处理下一个队列中的抽牌
processNextQueuedDraw(): ThunkAction

// 重置抽牌状态
resetDrawState(): PayloadAction
```

## UI组件

### DrawAnimation

抽牌动画组件，展示卡牌从牌库到手牌的动画。

**属性：**
- `playerId: string` - 抽牌玩家ID
- `cardIds: string[]` - 抽取的卡牌ID列表
- `reveal: boolean` - 是否显示卡牌内容
- `duration?: number` - 动画持续时间
- `onComplete?: () => void` - 动画完成回调
- `source?: DrawSource` - 抽牌来源

### DeckCounter

牌库计数器组件，显示牌库剩余卡牌数量。

**属性：**
- `playerId: string` - 玩家ID
- `count: number` - 牌库剩余数量
- `showAnimation?: boolean` - 是否显示数量变化动画
- `showLowWarning?: boolean` - 牌库数量低时是否显示警告
- `onDeckClick?: () => void` - 牌库点击回调

### DrawEffect

抽牌特效组件，显示不同类型抽牌的特殊效果。

**属性：**
- `type: 'normal' | 'ability' | 'special'` - 特效类型
- `position: Position` - 特效位置
- `duration?: number` - 特效持续时间
- `scale?: number` - 特效缩放
- `onComplete?: () => void` - 特效完成回调

## 使用示例

### 抽牌系统集成

```tsx
import { useDispatch, useSelector } from 'react-redux';
import { 
  drawCards, 
  viewTopDeck,
  selectIsDrawing,
  selectDeckSize,
  selectLastDrawnCards
} from '@features/current-match/card-draw';
import { 
  DrawAnimation, 
  DeckCounter 
} from '@features/current-match/card-draw/ui';
import { selectCurrentPlayerId, selectIsPlayerTurn } from '@features/current-match';

const PlayerDeckArea = () => {
  const dispatch = useDispatch();
  const currentPlayerId = useSelector(selectCurrentPlayerId);
  const deckSize = useSelector(state => selectDeckSize(state, currentPlayerId));
  const isDrawing = useSelector(selectIsDrawing);
  const lastDrawnCards = useSelector(selectLastDrawnCards);
  const isPlayerTurn = useSelector(selectIsPlayerTurn);
  
  // 处理抽牌操作
  const handleDrawCard = () => {
    if (isPlayerTurn && deckSize > 0) {
      dispatch(drawCards({ 
        playerId: currentPlayerId, 
        count: 1, 
        source: 'ABILITY', 
        animation: true 
      }));
    }
  };
  
  // 查看牌库顶牌
  const handleViewTopDeck = () => {
    dispatch(viewTopDeck(currentPlayerId));
  };
  
  return (
    <div className="player-deck-area">
      <DeckCounter
        playerId={currentPlayerId}
        count={deckSize}
        showAnimation={true}
        showLowWarning={deckSize <= 3}
        onDeckClick={handleViewTopDeck}
      />
      
      {/* 抽牌按钮（示例） */}
      {isPlayerTurn && (
        <button 
          onClick={handleDrawCard} 
          disabled={isDrawing || deckSize === 0}
          className="draw-card-button"
        >
          抽一张牌
        </button>
      )}
      
      {/* 抽牌动画 */}
      {isDrawing && lastDrawnCards.length > 0 && (
        <DrawAnimation
          playerId={currentPlayerId}
          cardIds={lastDrawnCards.map(card => card.cardId)}
          reveal={true}
          duration={800}
          source="ABILITY"
        />
      )}
    </div>
  );
};
```

### 抽牌事件监听

```tsx
import { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { selectDrawHistory } from '@features/current-match/card-draw';
import { playSound } from '@services/audio';

const DrawEventHandler = () => {
  const drawHistory = useSelector(selectDrawHistory);
  
  // 监听抽牌历史变化
  useEffect(() => {
    if (drawHistory.length > 0) {
      const latestDraw = drawHistory[drawHistory.length - 1];
      
      // 根据抽牌来源播放不同音效
      switch (latestDraw.source) {
        case 'TURN_START':
          playSound('card_draw_turn');
          break;
        case 'CARD_EFFECT':
          playSound('card_draw_effect');
          break;
        case 'SPECIAL':
          playSound('card_draw_special');
          break;
        default:
          playSound('card_draw_default');
      }
      
      // 如果抽牌后牌库为空，播放特殊警告音效
      if (latestDraw.count > 0 && getDeckSize(latestDraw.playerId) === 0) {
        playSound('deck_empty_warning');
      }
    }
  }, [drawHistory.length]);
  
  // 该组件不渲染任何UI
  return null;
};
```

## 与其他模块的关系

- 与**card-play**子模块协作，处理抽牌后可能出牌的卡牌
- 与**match-interactions**子模块交互，确保在正确的回合和阶段抽牌
- 更新**current-match**模块的核心游戏状态
- 可能触发**match-results**子模块中的游戏结果判定（如牌库耗尽）
- 使用**card**实体模块中定义的卡牌数据模型和展示组件
- 可能与**in-game-interactions**子模块交互，处理特殊抽牌交互

## 开发指南

1. 抽牌逻辑应在服务器端严格实现，客户端仅负责展示和初步验证
2. 抽牌动画应可配置速度，允许玩家根据偏好调整
3. 牌库耗尽检查应及时准确，确保游戏规则正确应用
4. 考虑网络延迟情况，提供适当的状态指示和回滚机制
5. 特殊抽牌效果（如"查看并选择"）应有明确的交互流程
6. 抽牌历史记录应足够详细，便于回放和调试
7. 优化多张连续抽牌的性能和视觉体验 