# 对战结果子模块 (Match Results Feature)

## 概述

对战结果子模块是当前对战模块的重要组成部分，专注于处理游戏对战结束后的各种逻辑和展示。该子模块负责确定对战胜负、计算和显示战斗统计数据、处理奖励分配、更新玩家排名和战绩，以及提供对战回顾和分享功能。作为玩家体验闭环的关键环节，对战结果子模块不仅展示游戏的最终状态，还提供有价值的反馈，帮助玩家分析和改进游戏策略。

## 文件结构

```
match-results/
├── index.ts            # 模块导出
├── model/              # 对战结果状态管理
│   ├── index.ts        # 模型导出
│   ├── store.ts        # 状态管理
│   ├── actions.ts      # 结果处理操作
│   ├── selectors.ts    # 状态选择器
│   └── types.ts        # 类型定义
├── ui/                 # 对战结果界面组件
│   ├── index.ts        # UI组件导出
│   ├── victory-defeat.tsx     # 胜负展示组件
│   ├── match-stats.tsx        # 对战统计组件
│   ├── rewards-display.tsx    # 奖励展示组件
│   └── match-history-card.tsx # 对战历史卡片
└── lib/                # 对战结果工具库
    ├── index.ts        # 工具库导出
    ├── stats-calculator.ts    # 统计数据计算
    ├── rank-calculator.ts     # 排名变化计算
    └── replay-generator.ts    # 回放生成工具
```

## 功能职责

### 对战结果判定

确定对战的胜负和结束方式：
- 基于生命值判定胜负
- 处理认输结果
- 处理超时判负
- 识别平局条件
- 处理异常结束（断连等）

### 统计数据分析

收集和计算对战统计信息：
- 回合数和对战时长
- 伤害输出和承受统计
- 卡牌使用情况分析
- 关键事件记录（如击杀）
- 资源利用效率（法力、卡牌）

### 奖励与进度系统

管理对战后的奖励分配：
- 经验值计算和等级提升
- 货币奖励确定
- 任务进度更新
- 成就解锁检查
- 特殊奖励（如连胜奖励）

### 排名与匹配评级

处理竞技排名的变化：
- 计算排名点数变动
- 更新玩家段位
- 记录匹配历史
- 调整匹配评分
- 处理赛季排名数据

### 对战回顾功能

提供对战回顾和分享工具：
- 生成对战摘要
- 保存关键回放片段
- 提供分享功能
- 生成对战报告
- 支持对战录像查看

## 数据模型

对战结果子模块的核心数据结构：

```typescript
// 对战结果状态
export interface MatchResultsState {
  matchId: string;
  isMatchEnded: boolean;
  winner: string | null;
  endReason: MatchEndReason;
  matchStats: MatchStats;
  rewards: MatchRewards;
  rankChanges: RankChanges;
  isLoadingResults: boolean;
  showResultsScreen: boolean;
  error: string | null;
}

// 对战结束原因
export enum MatchEndReason {
  VICTORY = 'VICTORY',           // 常规胜利（击败对手）
  SURRENDER = 'SURRENDER',       // 对手认输
  TIMEOUT = 'TIMEOUT',           // 对手超时
  DISCONNECTION = 'DISCONNECTION', // 对手断线
  DECK_OUT = 'DECK_OUT',         // 牌库耗尽
  DRAW = 'DRAW',                 // 平局
  ADMIN = 'ADMIN'                // 管理员结束
}

// 对战统计数据
export interface MatchStats {
  duration: number;              // 对战持续时间（秒）
  turnCount: number;             // 总回合数
  playerStats: Record<string, PlayerMatchStats>;
  keyEvents: MatchEvent[];       // 关键事件记录
}

// 玩家对战统计
export interface PlayerMatchStats {
  playerId: string;
  damageDealt: number;           // 造成的伤害
  damageTaken: number;           // 承受的伤害
  healingDone: number;           // 治疗量
  cardsPlayed: number;           // 使用的卡牌数
  cardsDrawn: number;            // 抽取的卡牌数
  manaSpent: number;             // 消耗的法力值
  creaturesSummoned: number;     // 召唤的生物数
  creaturesDestroyed: number;    // 摧毁的生物数
  specialStats: Record<string, any>; // 特殊统计数据
}

// 对战奖励
export interface MatchRewards {
  experience: number;            // 获得的经验值
  currency: Record<string, number>; // 获得的货币
  items: RewardItem[];           // 物品奖励
  questProgress: QuestProgress[]; // 任务进度
  achievementsUnlocked: string[]; // 解锁的成就
}

// 排名变化
export interface RankChanges {
  oldRank: PlayerRank;           // 旧排名信息
  newRank: PlayerRank;           // 新排名信息
  pointsChange: number;          // 点数变化
  tierChanged: boolean;          // 是否升降级
  streakBonus: number;           // 连胜/连败加成
}

// 对战事件
export interface MatchEvent {
  timestamp: number;             // 事件时间戳
  turnNumber: number;            // 事件发生的回合
  type: string;                  // 事件类型
  initiatorId: string;           // 事件发起者
  targetId?: string;             // 事件目标
  value?: number;                // 事件值
  metadata?: Record<string, any>; // 额外数据
}
```

## 主要操作

```typescript
// 处理对战结束
handleMatchEnd(payload: MatchEndPayload): ThunkAction

// 提交最终对战统计
submitMatchStats(): ThunkAction

// 更新对战奖励
updateRewards(rewards: MatchRewards): PayloadAction<MatchRewards>

// 更新排名变化
updateRankChanges(rankChanges: RankChanges): PayloadAction<RankChanges>

// 记录对战事件
recordMatchEvent(event: MatchEvent): PayloadAction<MatchEvent>

// 生成对战回放
generateMatchReplay(): ThunkAction

// 分享对战结果
shareMatchResults(options: ShareOptions): ThunkAction

// 保存对战历史
saveMatchToHistory(): ThunkAction

// 显示结果屏幕
showResultsScreen(): PayloadAction<boolean>

// 关闭结果屏幕
hideResultsScreen(): PayloadAction<boolean>
```

## UI组件

### VictoryDefeat

胜负结果展示组件，显示对战的最终结果。

**属性：**
- `isVictory: boolean` - 是否胜利
- `endReason: MatchEndReason` - 对战结束原因
- `opponent: PlayerInfo` - 对手信息
- `duration: number` - 对战持续时间（秒）
- `animationDelay?: number` - 动画延迟（毫秒）
- `onContinue?: () => void` - 继续按钮回调

### MatchStats

对战统计数据展示组件，显示详细的对战数据分析。

**属性：**
- `stats: MatchStats` - 对战统计数据
- `playerId: string` - 当前玩家ID
- `opponentId: string` - 对手玩家ID
- `showDetailedStats?: boolean` - 是否显示详细统计
- `compact?: boolean` - 是否使用紧凑模式
- `highlightThreshold?: number` - 高亮数据阈值

### RewardsDisplay

奖励展示组件，展示对战后获得的奖励。

**属性：**
- `rewards: MatchRewards` - 奖励数据
- `playerLevel: number` - 玩家当前等级
- `nextLevelXp: number` - 下一级所需经验
- `currentXp: number` - 当前经验值
- `showAnimation?: boolean` - 是否显示获得动画
- `onAllCollected?: () => void` - 全部收集回调

### MatchHistoryCard

对战历史卡片组件，用于在历史记录中展示对战信息。

**属性：**
- `matchId: string` - 对战ID
- `result: 'victory' | 'defeat' | 'draw'` - 对战结果
- `opponent: PlayerInfo` - 对手信息
- `date: Date` - 对战日期
- `duration: number` - 持续时间
- `deck: DeckInfo` - 使用的卡组
- `onClick?: () => void` - 点击卡片回调
- `showReplay?: boolean` - 是否显示回放按钮

## 使用示例

### 对战结果屏幕集成

```tsx
import { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { 
  handleMatchEnd, 
  showResultsScreen,
  selectIsMatchEnded,
  selectEndReason,
  selectWinner,
  selectMatchStats,
  selectRewards,
  selectRankChanges
} from '@features/current-match/match-results';
import { 
  VictoryDefeat, 
  MatchStats,
  RewardsDisplay
} from '@features/current-match/match-results/ui';
import { selectCurrentPlayerId, selectOpponentId } from '@features/current-match';
import { selectPlayerInfo } from '@entities/player';

const MatchResultsScreen = () => {
  const dispatch = useDispatch();
  const isMatchEnded = useSelector(selectIsMatchEnded);
  const endReason = useSelector(selectEndReason);
  const winner = useSelector(selectWinner);
  const matchStats = useSelector(selectMatchStats);
  const rewards = useSelector(selectRewards);
  const rankChanges = useSelector(selectRankChanges);
  const currentPlayerId = useSelector(selectCurrentPlayerId);
  const opponentId = useSelector(selectOpponentId);
  const opponentInfo = useSelector(state => selectPlayerInfo(state, opponentId));
  
  // 检测对战结束
  useEffect(() => {
    if (isMatchEnded) {
      // 显示结果屏幕（带延迟，让游戏结束动画播放完）
      const timer = setTimeout(() => {
        dispatch(showResultsScreen(true));
      }, 2000);
      
      return () => clearTimeout(timer);
    }
  }, [isMatchEnded, dispatch]);
  
  // 如果对战未结束，不渲染结果屏幕
  if (!isMatchEnded) return null;
  
  const isVictory = winner === currentPlayerId;
  
  return (
    <div className="match-results-screen">
      {/* 胜负结果展示 */}
      <VictoryDefeat 
        isVictory={isVictory}
        endReason={endReason}
        opponent={opponentInfo}
        duration={matchStats.duration}
        animationDelay={500}
      />
      
      {/* 制表符切换视图 */}
      <div className="results-tabs">
        <TabComponent
          tabs={[
            {
              label: '战斗统计',
              content: (
                <MatchStats 
                  stats={matchStats}
                  playerId={currentPlayerId}
                  opponentId={opponentId}
                  showDetailedStats={true}
                />
              )
            },
            {
              label: '奖励',
              content: (
                <RewardsDisplay 
                  rewards={rewards}
                  playerLevel={playerLevel}
                  nextLevelXp={nextLevelXp}
                  currentXp={currentXp}
                  showAnimation={true}
                />
              )
            },
            {
              label: '排名变化',
              content: (
                <RankChangeDisplay 
                  rankChanges={rankChanges}
                  isVictory={isVictory}
                />
              ),
              disabled: !isRankedMatch
            }
          ]}
        />
      </div>
      
      {/* 操作按钮 */}
      <div className="result-actions">
        <button onClick={() => dispatch(generateMatchReplay())}>
          保存回放
        </button>
        <button onClick={() => dispatch(shareMatchResults({ platform: 'all' }))}>
          分享结果
        </button>
        <button onClick={() => navigate('/home')}>
          返回主页
        </button>
        <button onClick={() => dispatch(findNewMatch())}>
          再来一局
        </button>
      </div>
    </div>
  );
};
```

### 对战历史记录集成

```tsx
import { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { 
  selectMatchHistory,
  loadMatchHistory
} from '@features/profile';
import { 
  MatchHistoryCard 
} from '@features/current-match/match-results/ui';

const MatchHistoryList = () => {
  const dispatch = useDispatch();
  const matchHistory = useSelector(selectMatchHistory);
  
  // 加载对战历史
  useEffect(() => {
    dispatch(loadMatchHistory());
  }, [dispatch]);
  
  // 查看对战详情
  const handleViewMatchDetails = (matchId) => {
    navigate(`/history/match/${matchId}`);
  };
  
  return (
    <div className="match-history-list">
      <h2>近期对战</h2>
      
      {matchHistory.length === 0 ? (
        <p>暂无对战记录</p>
      ) : (
        <div className="history-cards">
          {matchHistory.map(match => (
            <MatchHistoryCard 
              key={match.id}
              matchId={match.id}
              result={match.playerWon ? 'victory' : match.isDraw ? 'draw' : 'defeat'}
              opponent={match.opponent}
              date={new Date(match.endTime)}
              duration={match.duration}
              deck={match.deck}
              onClick={() => handleViewMatchDetails(match.id)}
              showReplay={true}
            />
          ))}
        </div>
      )}
    </div>
  );
};
```

## 与其他模块的关系

- 从**current-match**模块获取对战的最终状态
- 与**match-interactions**子模块交互，在认输或超时时触发结算
- 可能与**card-play**和**card-draw**子模块交互，获取详细的战局统计
- 与**player**实体模块协作，更新玩家统计和排名信息
- 与**profile**功能模块交互，更新玩家资料和对战历史
- 与**rewards**系统交互，处理对战奖励的分配和显示

## 开发指南

1. 对战结果判定应严格遵循游戏规则，确保公平公正
2. 结果屏幕应设计为逐步展示信息，避免一次呈现过多内容
3. 统计数据应有效地突出关键信息，帮助玩家理解对战结果
4. 奖励展示应有适当的视听反馈，增强玩家满足感
5. 排名变化应清晰易懂，特别是段位变化时提供明确的反馈
6. 考虑添加对战"亮点"功能，自动识别并保存精彩瞬间
7. 分享功能应生成易于理解的对战摘要，适合社交媒体分享
8. 对战数据应详细记录但精简存储，便于后续分析和回放
9. 结果屏幕应优雅处理各种异常结束情况，如断线和服务器错误 