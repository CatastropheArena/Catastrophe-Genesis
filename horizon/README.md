## 模块概述

horizon 模块是 catastrophe-genesis 项目的核心用户界面中心，提供了游戏内所有主要功能的访问入口。该模块整合了卡牌收藏、游戏匹配、质押系统、租赁市场和代币交换等关键功能，为玩家提供了一个统一的游戏管理界面。

## 核心功能

- **资产管理**: 提供玩家代币（Coins、Fragments、USDT）和卡牌资产的实时查看和管理功能
- **卡牌系统**: 包含卡牌收藏展示、卡牌合成、抽卡系统以及卡牌升级功能
- **经济系统**: 集成代币交换、卡牌租赁市场和质押池等 GameFi 核心经济功能
- **游戏匹配**: 提供不同级别（初级、中级、高级）的游戏匹配功能
- **质押系统**: 支持卡牌质押获取收益，包含不同稀有度的质押池和收益率
- **租赁市场**: 允许玩家租借和出租卡牌，支持灵活的租期和使用次数设置

## 用户旅程

### 新玩家入门旅程

```mermaid
journey
    title 新玩家入门流程
    section 初始化
        进入游戏: 5: 新玩家
        查看新手引导: 5: 新玩家
        领取新手奖励: 5: 新玩家
    section 基础玩法
        购买第一个卡包: 4: 新玩家
        打开卡包: 5: 新玩家
        参与初级对局: 3: 新玩家
    section 进阶操作
        尝试卡牌合成: 3: 新玩家
        探索租赁市场: 2: 新玩家
        了解质押系统: 2: 新玩家
```

### 资深玩家日常旅程

```mermaid
journey
    title 资深玩家日常活动
    section 日常管理
        检查资产状态: 5: 资深玩家
        收取质押收益: 5: 资深玩家
        更新租赁列表: 4: 资深玩家
    section 游戏活动
        参与高级对局: 5: 资深玩家
        卡牌升级合成: 4: 资深玩家
        市场交易: 4: 资深玩家
    section 投资活动
        调整质押策略: 3: 资深玩家
        优化租赁收益: 4: 资深玩家
        代币交换: 3: 资深玩家
```

## 用例图

### 核心功能用例

```mermaid
graph TB
    User((玩家))
    
    subgraph 资产管理
        A1[查看资产]
        A2[资产转换]
        A3[收益提取]
    end
    
    subgraph 卡牌系统
        B1[查看收藏]
        B2[合成升级]
        B3[购买卡包]
        B4[打开卡包]
    end
    
    subgraph 游戏系统
        C1[加入对局]
        C2[选择难度]
        C3[获取奖励]
    end
    
    User --> A1
    User --> A2
    User --> A3
    User --> B1
    User --> B2
    User --> B3
    User --> B4
    User --> C1
    User --> C2
    User --> C3
```

### 经济系统用例

```mermaid
graph TB
    User((玩家))
    
    subgraph 质押系统
        D1[质押卡牌]
        D2[查看收益]
        D3[提取收益]
    end
    
    subgraph 租赁市场
        E1[出租卡牌]
        E2[租用卡牌]
        E3[管理租约]
    end
    
    subgraph 代币交换
        F1[代币兑换]
        F2[查看汇率]
        F3[交易历史]
    end
    
    User --> D1
    User --> D2
    User --> D3
    User --> E1
    User --> E2
    User --> E3
    User --> F1
    User --> F2
    User --> F3
```

## 对象结构

### 资产系统类图

```mermaid
classDiagram
    class Assets {
        +number coins
        +number fragments
        +number usdt
        +CardItem[] cards
    }
    
    class CardItem {
        +number|string id
        +string name
        +string rarity
        +string image
        +number count
        +string status
        +number? usesLeft
        +string? poolShare
        +number? expiresIn
    }
    
    class RentalCard {
        +number|string id
        +string name
        +string rarity
        +string image
        +number poolSize
        +number rate
    }
    
    class StakingPool {
        +number|string id
        +string name
        +string rarity
        +string image
        +number poolSize
        +string apr
        +number rate
    }
    
    Assets "1" *-- "many" CardItem
    CardItem <|-- RentalCard
    CardItem <|-- StakingPool
```

### 游戏系统类图

```mermaid
classDiagram
    class GameMatch {
        +number id
        +string name
        +string level
        +number entryFee
        +number currentPlayers
        +number maxPlayers
        +number rewards
        +string bgClass
        +string badgeClass
    }
    
    class CardPack {
        +number id
        +string name
        +number price
        +string image
        +string description
        +DropRates dropRates
    }
    
    class DropRates {
        +number Common
        +number Uncommon
        +number Rare
        +number Legendary
    }
    
    class DrawHistoryItem {
        +number|string id
        +string packName
        +CardItem card
        +string timestamp
    }
    
    CardPack "1" *-- "1" DropRates
    DrawHistoryItem "1" *-- "1" CardItem
```

## 关键组件

### 页面组件
- **page.tsx**: 主面板页面组件，整合所有子功能模块，管理全局状态和用户交互逻辑

### 功能组件
- **card-collection.tsx**: 卡牌收藏展示组件，管理玩家拥有的卡牌
- **card-synthesis-gacha.tsx**: 卡牌合成和抽卡系统组件，处理卡牌升级和获取逻辑
- **card-staking-pools.tsx**: 质押池组件，管理卡牌质押和收益计算
- **card-rental-marketplace.tsx**: 租赁市场组件，处理卡牌租赁相关功能
- **game-matches.tsx**: 游戏匹配组件，展示可用游戏房间和匹配功能
- **exchange.tsx**: 代币交换组件，处理不同代币间的兑换功能

### 辅助组件
- **header.tsx**: 顶部导航栏组件，显示用户信息和主要功能入口
- **welcome.tsx**: 欢迎界面组件，展示用户状态和快速操作入口
- **dialog-modal.tsx**: 通用对话框组件，用于用户交互确认
- **drawn-card.tsx**: 抽卡结果展示组件，显示新获得的卡牌

## 依赖关系

### 内部依赖
- **@/app/types**: 提供全局类型定义，包括资产、卡牌、对话框等接口
- **@/components/ui**: 使用项目通用 UI 组件库
- **@/hooks**: 使用自定义 hooks，如 useMobile 用于响应式适配

### 外部依赖
- **React**: 核心框架依赖
- **Next.js**: 应用框架，提供路由和客户端渲染支持
- **Lucide React**: 图标库
- **Tailwind CSS**: 样式框架

## 使用示例

```typescript
// 初始化 Dashboard 页面
import DashboardPage from './dashboard/page'

// 渲染主面板
<DashboardPage />

// 处理卡牌质押
const handleStakeCard = (card: StakingPool) => {
  // 检查用户拥有的卡牌
  const userCard = assets.cards.find(c => c.name === card.name)
  if (!userCard || userCard.count === 0) {
    setDialog({
      open: true,
      title: "质押失败",
      description: "您没有足够的卡牌进行质押",
      type: "error"
    })
    return
  }
  // 打开质押输入对话框
  setDialog({
    open: true,
    title: "质押卡牌",
    description: `请输入要质押的 ${card.name} 数量`,
    type: "stakeInput",
    confirmAction: (amount) => confirmStakeCard(card, amount!, userCard)
  })
}
```

## 架构说明

Dashboard 模块采用组件化架构，通过状态提升和属性传递实现组件间通信。主要数据流如下：

```mermaid
graph TD
    A[Dashboard Page] --> B[资产状态管理]
    B --> C[卡牌收藏]
    B --> D[质押系统]
    B --> E[租赁市场]
    B --> F[游戏匹配]
    B --> G[代币交换]
    C --> H[卡牌合成]
    C --> I[抽卡系统]
    D --> J[收益计算]
    E --> K[租赁管理]
```

- 采用 Next.js App Router 架构
- 使用客户端组件确保实时交互体验
- 实现模块化设计，便于功能扩展和维护
- 统一的状态管理和事件处理机制
- 响应式设计支持多设备适配 