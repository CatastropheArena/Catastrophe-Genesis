# 出牌功能子模块 (Card Play Feature)

## 概述

出牌功能子模块是当前对战模块的关键组成部分，专注于处理玩家出牌相关的所有操作和状态管理。该子模块负责实现卡牌从手牌到场上的完整流程，包括出牌验证、目标选择、出牌动画、卡牌效果触发和结算等。作为对战核心玩法的一部分，出牌功能直接影响游戏的交互体验和策略深度。

## 文件结构

```
card-play/
├── index.ts            # 模块导出
├── model/              # 出牌状态管理
│   ├── index.ts        # 模型导出
│   ├── actions.ts      # 出牌相关操作
│   ├── selectors.ts    # 状态选择器
│   ├── types.ts        # 类型定义
│   └── reducer.ts      # 出牌状态reducer
└── ui/                 # 出牌界面组件
    ├── index.ts        # UI组件导出
    ├── card-target-selector.tsx   # 卡牌目标选择器
    ├── playable-card.tsx          # 可出牌卡牌组件
    └── play-animation.tsx         # 出牌动画组件
```

## 功能职责

### 出牌验证与规则检查

验证玩家的出牌操作是否合法：
- 检查玩家是否处于可出牌状态
- 验证玩家的法力值是否足够支付卡牌费用
- 验证场上是否有足够空间放置卡牌
- 检查卡牌特定的出牌条件是否满足
- 判断目标选择是否符合卡牌要求

### 目标选择管理

处理需要选择目标的卡牌：
- 识别卡牌的有效目标类型（生物、玩家、场地等）
- 高亮显示可选择的合法目标
- 管理多目标卡牌的目标选择流程
- 提供目标选择的取消机制
- 验证已选目标的合法性

### 出牌动画与视觉反馈

提供流畅的出牌视觉体验：
- 卡牌从手牌到场上的移动动画
- 卡牌打出时的特效展示
- 与目标交互的视觉效果
- 场上卡牌布局的实时调整
- 出牌成功或失败的反馈提示

### 出牌结果处理

管理出牌后的状态更新和效果触发：
- 扣除玩家法力值
- 将卡牌移至适当区域（场上、墓地等）
- 触发卡牌的"入场"效果
- 更新游戏状态（如回合阶段变化）
- 记录出牌历史用于重放和效果追踪

## 数据模型

出牌功能子模块的核心数据结构：

```typescript
// 出牌状态
export interface CardPlayState {
  isSelectingTarget: boolean;
  selectedCardId: string | null;
  validTargets: Target[];
  selectedTargets: Target[];
  requiredTargetCount: number;
  isProcessingPlay: boolean;
  lastPlayedCard: PlayedCardInfo | null;
  error: string | null;
}

// 目标信息
export interface Target {
  id: string;
  type: TargetType;
  position?: Position;
}

// 目标类型枚举
export enum TargetType {
  PLAYER = 'PLAYER',
  CREATURE = 'CREATURE',
  SPELL = 'SPELL',
  FIELD_POSITION = 'FIELD_POSITION',
  HAND_CARD = 'HAND_CARD'
}

// 已出牌信息
export interface PlayedCardInfo {
  cardId: string;
  playerId: string;
  timestamp: number;
  targets: Target[];
  position?: Position;
  manaCost: number;
}
```

## 主要操作

```typescript
// 开始出牌流程
initiateCardPlay(cardId: string): PayloadAction<string>

// 选择卡牌目标
selectCardTarget(target: Target): PayloadAction<Target>

// 取消目标选择
cancelTargetSelection(): PayloadAction

// 确认出牌（提交服务器）
confirmCardPlay(): ThunkAction

// 处理出牌响应
handlePlayResponse(response: PlayResponse): PayloadAction<PlayResponse>

// 重置出牌状态
resetCardPlayState(): PayloadAction
```

## UI组件

### CardTargetSelector

卡牌目标选择器组件，用于高亮和选择卡牌目标。

**属性：**
- `validTargets: Target[]` - 有效目标列表
- `selectedTargets: Target[]` - 已选目标列表
- `requiredTargetCount: number` - 需要选择的目标数量
- `onSelect: (target: Target) => void` - 选择目标回调
- `onCancel: () => void` - 取消选择回调
- `onConfirm: () => void` - 确认选择回调
- `targetingType: TargetingType` - 目标选择类型

### PlayableCard

可出牌卡牌组件，增强手牌中可以出牌的卡牌。

**属性：**
- `card: Card` - 卡牌数据
- `isPlayable: boolean` - 是否可出牌
- `onPlay: () => void` - 出牌回调
- `showPlayableHighlight?: boolean` - 是否显示可出牌高亮
- `playRequiresTarget?: boolean` - 是否需要选择目标

### PlayAnimation

卡牌出牌动画组件，展示从手牌到场上的动画效果。

**属性：**
- `card: Card` - 卡牌数据
- `startPosition: Position` - 起始位置
- `endPosition: Position` - 结束位置
- `duration?: number` - 动画持续时间
- `onComplete?: () => void` - 动画完成回调
- `effectType?: 'normal' | 'spell' | 'creature'` - 特效类型

## 使用示例

### 集成出牌系统

```tsx
import { useDispatch, useSelector } from 'react-redux';
import { 
  initiateCardPlay, 
  selectCardTarget,
  confirmCardPlay,
  cancelTargetSelection,
  selectIsSelectingTarget,
  selectSelectedCardId,
  selectValidTargets,
  selectSelectedTargets,
  selectRequiredTargetCount
} from '@features/current-match/card-play';
import { 
  CardTargetSelector, 
  PlayableCard 
} from '@features/current-match/card-play/ui';
import { selectHandCards } from '@features/current-match';

const PlayerHand = () => {
  const dispatch = useDispatch();
  const handCards = useSelector(selectHandCards);
  const isSelectingTarget = useSelector(selectIsSelectingTarget);
  const selectedCardId = useSelector(selectSelectedCardId);
  const validTargets = useSelector(selectValidTargets);
  const selectedTargets = useSelector(selectSelectedTargets);
  const requiredTargetCount = useSelector(selectRequiredTargetCount);
  
  // 处理出牌操作
  const handleCardPlay = (cardId) => {
    dispatch(initiateCardPlay(cardId));
  };
  
  // 处理目标选择
  const handleTargetSelect = (target) => {
    dispatch(selectCardTarget(target));
    
    // 如果已选足够的目标，自动确认
    if (selectedTargets.length + 1 >= requiredTargetCount) {
      dispatch(confirmCardPlay());
    }
  };
  
  // 取消出牌
  const handleCancel = () => {
    dispatch(cancelTargetSelection());
  };
  
  return (
    <div className="player-hand">
      {/* 渲染手牌卡牌 */}
      {handCards.map(card => (
        <PlayableCard
          key={card.id}
          card={card}
          isPlayable={card.isPlayable}
          playRequiresTarget={card.requiresTarget}
          onPlay={() => handleCardPlay(card.id)}
          showPlayableHighlight={true}
        />
      ))}
      
      {/* 目标选择器 */}
      {isSelectingTarget && (
        <CardTargetSelector
          validTargets={validTargets}
          selectedTargets={selectedTargets}
          requiredTargetCount={requiredTargetCount}
          onSelect={handleTargetSelect}
          onCancel={handleCancel}
          onConfirm={() => dispatch(confirmCardPlay())}
          targetingType="single"
        />
      )}
    </div>
  );
};
```

### 监听出牌事件

```tsx
import { useEffect } from 'react';
import { useSelector } from 'react-redux';
import { selectLastPlayedCard } from '@features/current-match/card-play';
import { PlayAnimation } from '@features/current-match/card-play/ui';
import { selectCardById } from '@entities/card';

const CardPlayEffectsHandler = () => {
  const lastPlayedCard = useSelector(selectLastPlayedCard);
  
  // 监听最近出牌变化，播放相应动画
  useEffect(() => {
    if (lastPlayedCard) {
      // 可以播放音效或其他效果
      const audioFx = getCardPlaySound(lastPlayedCard.cardId);
      if (audioFx) {
        playSound(audioFx);
      }
    }
  }, [lastPlayedCard]);
  
  if (!lastPlayedCard) return null;
  
  const card = useSelector(state => selectCardById(state, lastPlayedCard.cardId));
  
  return (
    <PlayAnimation
      card={card}
      startPosition={lastPlayedCard.startPosition}
      endPosition={lastPlayedCard.position}
      duration={800}
      effectType={card.type === 'SPELL' ? 'spell' : 'creature'}
    />
  );
};
```

## 与其他模块的关系

- 与**card-draw**子模块协作，处理从牌库抽取的卡牌
- 与**match-interactions**子模块交互，确保在正确的回合和阶段出牌
- 使用**in-game-interactions**子模块提供的交互框架进行目标选择
- 更新**current-match**模块的核心游戏状态
- 可能触发**match-results**子模块中的游戏结果判定（如通过出牌获胜）
- 使用**card**实体模块中定义的卡牌数据模型和展示组件

## 开发指南

1. 出牌验证逻辑应在客户端和服务器端双重实现，以防作弊
2. 目标选择系统应具有灵活性，能够处理各种类型的目标条件
3. 出牌动画应流畅且可配置，同时考虑不同设备性能的差异
4. 在网络延迟情况下，应提供适当的视觉反馈和回滚机制
5. 考虑添加出牌预览功能，让玩家在出牌前查看可能的结果
6. 异常情况处理应提供清晰的反馈，例如法力不足或目标无效
7. 卡牌特效应适当考虑性能影响，提供不同质量级别的设置选项 