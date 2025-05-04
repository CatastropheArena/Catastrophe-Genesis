# 游戏内交互子模块 (In-Game Interactions Feature)

## 概述

游戏内交互子模块是当前对战模块的核心交互层，专注于处理玩家在对战过程中与游戏元素的各种交互行为。该子模块提供统一的交互框架，包括选择、拖放、鼠标悬停、右键菜单等操作，为卡牌游戏的各种交互需求提供基础支持。作为游戏体验的重要组成部分，游戏内交互子模块负责创建流畅、直观且一致的用户交互体验。

## 文件结构

```
in-game-interactions/
├── index.ts            # 模块导出
├── model/              # 交互状态管理
│   ├── index.ts        # 模型导出
│   ├── store.ts        # 状态管理
│   ├── actions.ts      # 交互操作
│   ├── selectors.ts    # 状态选择器
│   └── types.ts        # 类型定义
├── ui/                 # 交互界面组件
│   ├── index.ts        # UI组件导出
│   ├── drag-layer.tsx  # 拖放层组件
│   ├── hover-card.tsx  # 悬停卡牌预览
│   ├── context-menu.tsx # 右键菜单
│   └── selection-indicator.tsx # 选择指示器
└── lib/                # 交互工具库
    ├── index.ts        # 工具库导出
    ├── drag-drop.ts    # 拖放工具函数
    ├── hover.ts        # 悬停检测工具
    └── selection.ts    # 选择工具函数
```

## 功能职责

### 交互状态管理

管理游戏内各种交互状态：
- 跟踪当前选中的游戏元素
- 管理拖放操作的状态和目标
- 追踪鼠标悬停的元素
- 控制右键菜单的显示和位置
- 记录交互历史用于撤销和重做

### 卡牌交互处理

提供卡牌特定的交互功能：
- 卡牌拖放移动和放置
- 卡牌悬停预览和详情
- 卡牌点击和双击行为
- 卡牌右键菜单选项
- 多卡牌选择操作

### 游戏板交互

管理游戏板上的交互：
- 场地位置选择和高亮
- 战场区域划分和识别
- 回合计时器交互
- 游戏阶段指示器交互
- 特殊交互区域处理（墓地、放逐区等）

### 交互反馈系统

提供视觉和音频反馈：
- 交互动作的视觉提示
- 交互成功或失败的反馈
- 交互限制的提示信息
- 操作引导和教学提示
- 配置化的反馈系统

## 数据模型

游戏内交互子模块的核心数据结构：

```typescript
// 游戏内交互状态
export interface InGameInteractionsState {
  selectedElement: InteractiveElement | null;
  hoveredElement: InteractiveElement | null;
  isDragging: boolean;
  dragSource: InteractiveElement | null;
  dragTarget: InteractiveElement | null;
  contextMenu: ContextMenuState | null;
  interactionMode: InteractionMode;
  interactionHistory: InteractionEvent[];
  lockedInteractions: boolean;
}

// 交互元素
export interface InteractiveElement {
  id: string;
  type: InteractiveElementType;
  position?: Position;
  ownerId?: string;
  metadata?: Record<string, any>;
}

// 交互元素类型枚举
export enum InteractiveElementType {
  CARD = 'CARD',
  FIELD_POSITION = 'FIELD_POSITION',
  PLAYER = 'PLAYER',
  DECK = 'DECK',
  GRAVEYARD = 'GRAVEYARD',
  TURN_BUTTON = 'TURN_BUTTON',
  ABILITY = 'ABILITY'
}

// 交互模式枚举
export enum InteractionMode {
  NORMAL = 'NORMAL',
  CARD_PLACEMENT = 'CARD_PLACEMENT',
  TARGET_SELECTION = 'TARGET_SELECTION',
  SPECTATING = 'SPECTATING',
  ANIMATION_PLAYING = 'ANIMATION_PLAYING'
}

// 右键菜单状态
export interface ContextMenuState {
  position: { x: number, y: number };
  options: ContextMenuOption[];
  targetElement: InteractiveElement;
}

// 右键菜单选项
export interface ContextMenuOption {
  id: string;
  label: string;
  icon?: string;
  action: () => void;
  disabled?: boolean;
  subOptions?: ContextMenuOption[];
}

// 交互事件记录
export interface InteractionEvent {
  timestamp: number;
  type: InteractionEventType;
  element: InteractiveElement;
  metadata?: Record<string, any>;
}

// 交互事件类型枚举
export enum InteractionEventType {
  SELECT = 'SELECT',
  DRAG_START = 'DRAG_START',
  DRAG_END = 'DRAG_END',
  HOVER_START = 'HOVER_START',
  HOVER_END = 'HOVER_END',
  CONTEXT_MENU = 'CONTEXT_MENU',
  ACTION = 'ACTION'
}
```

## 主要操作

```typescript
// 选择游戏元素
selectElement(element: InteractiveElement): PayloadAction<InteractiveElement>

// 取消选择
clearSelection(): PayloadAction

// 开始拖拽操作
startDrag(element: InteractiveElement): PayloadAction<InteractiveElement>

// 设置拖拽目标
setDragTarget(target: InteractiveElement | null): PayloadAction<InteractiveElement | null>

// 结束拖拽操作
endDrag(success: boolean): PayloadAction<boolean>

// 设置悬停元素
setHoveredElement(element: InteractiveElement | null): PayloadAction<InteractiveElement | null>

// 显示右键菜单
showContextMenu(options: ContextMenuOptions): PayloadAction<ContextMenuOptions>

// 隐藏右键菜单
hideContextMenu(): PayloadAction

// 设置交互模式
setInteractionMode(mode: InteractionMode): PayloadAction<InteractionMode>

// 锁定/解锁交互
setInteractionLock(locked: boolean): PayloadAction<boolean>
```

## UI组件

### DragLayer

拖拽层组件，处理游戏内元素的拖拽视觉效果。

**属性：**
- `isDragging: boolean` - 是否正在拖拽
- `dragSource: InteractiveElement | null` - 拖拽源元素
- `dragTarget: InteractiveElement | null` - 拖拽目标
- `offset: { x: number, y: number }` - 鼠标偏移量
- `renderItem: (element: InteractiveElement) => ReactNode` - 自定义渲染函数

### HoverCard

卡牌悬停预览组件，显示卡牌详细信息。

**属性：**
- `card: Card` - 卡牌数据
- `position: { x: number, y: number }` - 显示位置
- `showStats: boolean` - 是否显示统计数据
- `showDescription: boolean` - 是否显示描述
- `scale?: number` - 缩放比例
- `showRelatedCards?: boolean` - 是否显示相关卡牌

### ContextMenu

右键菜单组件，提供上下文操作选项。

**属性：**
- `position: { x: number, y: number }` - 菜单位置
- `options: ContextMenuOption[]` - 菜单选项
- `onClose: () => void` - 关闭回调
- `maxHeight?: number` - 最大高度
- `theme?: 'light' | 'dark'` - 菜单主题

### SelectionIndicator

选择指示器组件，高亮显示选中的元素。

**属性：**
- `element: InteractiveElement` - 选中的元素
- `type: 'primary' | 'secondary' | 'target'` - 指示器类型
- `pulseEffect?: boolean` - 是否显示脉冲效果
- `duration?: number` - 显示持续时间
- `color?: string` - 指示器颜色

## 使用示例

### 拖放交互集成

```tsx
import { useDispatch, useSelector } from 'react-redux';
import { 
  selectElement, 
  startDrag,
  setDragTarget,
  endDrag,
  selectIsDragging,
  selectDragSource,
  selectDragTarget,
  selectInteractionMode
} from '@features/current-match/in-game-interactions';
import { 
  DragLayer 
} from '@features/current-match/in-game-interactions/ui';
import { Card } from '@entities/card';

// 拖放源组件
const DraggableCard = ({ card }) => {
  const dispatch = useDispatch();
  const isDragging = useSelector(selectIsDragging);
  const dragSource = useSelector(selectDragSource);
  const interactionMode = useSelector(selectInteractionMode);
  
  // 处理拖拽开始
  const handleDragStart = (e) => {
    e.preventDefault();
    
    if (interactionMode !== 'NORMAL') return;
    
    const element = {
      id: card.id,
      type: 'CARD',
      ownerId: card.ownerId,
      metadata: { cardType: card.type }
    };
    
    dispatch(startDrag(element));
  };
  
  // 渲染卡牌
  return (
    <div 
      className={`draggable-card ${isDragging && dragSource?.id === card.id ? 'dragging' : ''}`}
      onMouseDown={handleDragStart}
      onClick={() => dispatch(selectElement({
        id: card.id,
        type: 'CARD'
      }))}
    >
      <Card card={card} />
    </div>
  );
};

// 拖放目标组件
const DropTarget = ({ position, isValid }) => {
  const dispatch = useDispatch();
  const isDragging = useSelector(selectIsDragging);
  const dragTarget = useSelector(selectDragTarget);
  
  // 处理拖拽进入
  const handleDragEnter = () => {
    if (!isDragging || !isValid) return;
    
    dispatch(setDragTarget({
      id: `position-${position.x}-${position.y}`,
      type: 'FIELD_POSITION',
      position
    }));
  };
  
  // 处理拖拽离开
  const handleDragLeave = () => {
    if (!isDragging) return;
    
    if (dragTarget?.id === `position-${position.x}-${position.y}`) {
      dispatch(setDragTarget(null));
    }
  };
  
  // 处理放置
  const handleDrop = () => {
    if (!isDragging || !isValid) return;
    
    // 确认放置
    dispatch(endDrag(true));
    
    // 在这里可以触发卡牌放置的实际操作
    // 例如：dispatch(playCard(dragSource.id, position));
  };
  
  return (
    <div 
      className={`drop-target ${
        isDragging && dragTarget?.id === `position-${position.x}-${position.y}` ? 'active' : ''
      } ${isValid ? 'valid' : 'invalid'}`}
      onMouseEnter={handleDragEnter}
      onMouseLeave={handleDragLeave}
      onMouseUp={handleDrop}
    />
  );
};

// 游戏区域组件
const GameArea = () => {
  const dispatch = useDispatch();
  const isDragging = useSelector(selectIsDragging);
  const dragSource = useSelector(selectDragSource);
  
  // 处理拖拽取消
  const handleDragCancel = () => {
    if (isDragging) {
      dispatch(endDrag(false));
    }
  };
  
  // 自定义拖拽渲染
  const renderDragItem = (element) => {
    if (element.type === 'CARD') {
      // 从状态中获取卡牌数据
      const card = getCardById(element.id);
      return <Card card={card} isDragging={true} />;
    }
    return null;
  };
  
  return (
    <div 
      className="game-area"
      onMouseUp={handleDragCancel}
    >
      {/* 游戏区域内容 */}
      
      {/* 拖拽层 */}
      <DragLayer 
        isDragging={isDragging}
        dragSource={dragSource}
        renderItem={renderDragItem}
      />
    </div>
  );
};
```

### 右键菜单和悬停交互

```tsx
import { useState, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { 
  setHoveredElement, 
  showContextMenu,
  hideContextMenu,
  selectHoveredElement,
  selectContextMenu
} from '@features/current-match/in-game-interactions';
import { 
  HoverCard,
  ContextMenu 
} from '@features/current-match/in-game-interactions/ui';
import { selectCardById } from '@entities/card';

const InteractiveCard = ({ cardId, playerId }) => {
  const dispatch = useDispatch();
  const card = useSelector(state => selectCardById(state, cardId));
  const hoveredElement = useSelector(selectHoveredElement);
  const contextMenu = useSelector(selectContextMenu);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  
  // 处理悬停开始
  const handleMouseEnter = () => {
    dispatch(setHoveredElement({
      id: cardId,
      type: 'CARD',
      ownerId: playerId
    }));
  };
  
  // 处理悬停结束
  const handleMouseLeave = () => {
    if (hoveredElement?.id === cardId) {
      dispatch(setHoveredElement(null));
    }
  };
  
  // 处理右键菜单
  const handleContextMenu = (e) => {
    e.preventDefault();
    
    setPosition({ x: e.clientX, y: e.clientY });
    
    const menuOptions = [
      {
        id: 'view',
        label: '查看详情',
        icon: 'eye',
        action: () => showCardDetails(cardId)
      },
      {
        id: 'info',
        label: '卡牌历史',
        icon: 'history',
        action: () => showCardHistory(cardId)
      }
    ];
    
    // 根据卡牌类型添加特定选项
    if (card.type === 'CREATURE') {
      menuOptions.push({
        id: 'attack',
        label: '攻击',
        icon: 'sword',
        action: () => initiateAttack(cardId),
        disabled: !canAttackWithCard(cardId)
      });
    }
    
    dispatch(showContextMenu({
      position: { x: e.clientX, y: e.clientY },
      options: menuOptions,
      targetElement: {
        id: cardId,
        type: 'CARD',
        ownerId: playerId
      }
    }));
  };
  
  return (
    <div 
      className="interactive-card"
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onContextMenu={handleContextMenu}
    >
      <Card card={card} />
      
      {/* 当前卡片的右键菜单 */}
      {contextMenu && contextMenu.targetElement.id === cardId && (
        <ContextMenu 
          position={contextMenu.position}
          options={contextMenu.options}
          onClose={() => dispatch(hideContextMenu())}
        />
      )}
    </div>
  );
};

// 全局悬停卡片预览组件
const GlobalHoverPreview = () => {
  const hoveredElement = useSelector(selectHoveredElement);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  
  // 跟踪鼠标位置
  useEffect(() => {
    const handleMouseMove = (e) => {
      setPosition({ x: e.clientX + 15, y: e.clientY + 15 });
    };
    
    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, []);
  
  if (!hoveredElement || hoveredElement.type !== 'CARD') return null;
  
  const card = getCardById(hoveredElement.id);
  if (!card) return null;
  
  return (
    <HoverCard 
      card={card}
      position={position}
      showStats={true}
      showDescription={true}
      showRelatedCards={true}
    />
  );
};
```

## 与其他模块的关系

- 与**card-play**子模块协作，处理卡牌的拖放出牌操作
- 与**card-draw**子模块交互，支持卡牌抽取后的视觉交互
- 为**match-interactions**子模块提供底层交互框架
- 与**match-results**子模块协作，在游戏结束时锁定交互
- 与**current-match**模块的核心游戏状态交互
- 使用**card**实体模块中定义的卡牌数据模型和展示组件

## 开发指南

1. 交互系统应具有高度一致性，不同游戏元素应遵循相似的交互模式
2. 视觉反馈应及时且明确，确保玩家了解其交互状态
3. 各种交互限制（如不能拖放的元素）应有清晰的视觉区分
4. 交互系统应支持键盘操作，提高可访问性
5. 性能优化应作为优先事项，特别是拖拽和悬停等频繁的操作
6. 交互统计和分析应被纳入，以便优化最常用的交互方式
7. 应考虑不同设备的交互差异，尤其是触摸设备的特殊需求
8. 错误处理机制应健壮，防止交互错误导致游戏状态异常
9. 交互的响应性是关键，用户操作与系统响应之间的延迟应最小化 