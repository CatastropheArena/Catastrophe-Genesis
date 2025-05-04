# 大厅设置子模块 (Lobby Settings Feature)

## 概述

大厅设置子模块是当前大厅功能的重要组成部分，专注于管理和配置游戏大厅的各种设置选项。该子模块负责处理游戏模式选择、自定义规则配置、玩家数量限制、地图选择以及其他与大厅设置相关的功能。它为大厅主机提供了灵活且全面的游戏设置管理界面，同时也为普通玩家提供了查看当前配置的能力。

## 文件结构

```
lobby-settings/
├── index.ts            # 模块导出
├── model/              # 设置状态管理
│   ├── index.ts        # 模型导出
│   ├── actions.ts      # 设置操作
│   ├── selectors.ts    # 设置选择器
│   └── validation.ts   # 设置验证逻辑
└── ui/                 # 设置界面组件
    ├── index.ts        # UI组件导出
    ├── settings-panel.tsx      # 设置面板组件
    ├── game-mode-selector.tsx  # 游戏模式选择器
    ├── custom-rules-form.tsx   # 自定义规则表单
    └── map-selector.tsx        # 地图选择器
```

## 功能职责

### 设置状态管理

管理大厅设置的状态和操作：
- 保存和获取当前设置状态
- 处理设置变更和验证
- 管理设置保存和应用流程
- 处理设置冲突和限制
- 提供派生设置信息（如基于游戏模式的默认值）

### 游戏模式配置

管理不同游戏模式的选择和配置：
- 支持不同游戏模式（休闲、竞技、自定义等）
- 根据模式自动调整相关设置
- 提供模式特定设置选项
- 管理模式限制和要求
- 验证模式兼容性

### 自定义规则设置

提供游戏规则的自定义选项：
- 初始生命值和资源设置
- 回合和时间限制配置
- 特殊规则启用/禁用
- 胜利条件调整
- 牌组和卡牌限制

### 游戏环境配置

管理游戏环境和其他设置：
- 地图/场景选择
- 队伍大小和分配
- 玩家数量限制
- 观战者设置
- 隐私和安全选项

## 数据模型

大厅设置子模块的核心数据结构：

```typescript
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

// 自定义规则
export interface CustomRules {
  startingLife: number;
  startingCards: number;
  maxHandSize: number;
  turnTimeLimit: number;
  specialRules: string[];
}

// 地图信息
export interface MapInfo {
  id: string;
  name: string;
  description: string;
  thumbnail: string;
  supportedModes: GameMode[];
  maxPlayers: number;
  recommendedPlayers: number;
}

// 设置验证结果
export interface SettingsValidationResult {
  isValid: boolean;
  errors: Record<string, string>;
  warnings: Record<string, string>;
}
```

## 主要操作

```typescript
// 更新设置
updateSettings(settings: Partial<LobbySettings>): ThunkAction

// 加载默认设置
loadDefaultSettings(gameMode: GameMode): ThunkAction

// 保存设置到服务器
saveSettings(): ThunkAction

// 验证设置
validateSettings(settings: LobbySettings): SettingsValidationResult

// 更新游戏模式
setGameMode(gameMode: GameMode): ThunkAction

// 更新自定义规则
updateCustomRules(rules: Partial<CustomRules>): ThunkAction

// 选择地图
selectMap(mapId: string): ThunkAction

// 设置最大玩家数
setMaxPlayers(count: number): ThunkAction

// 切换私密状态
togglePrivate(isPrivate: boolean): ThunkAction

// 重置设置为默认值
resetSettings(): ThunkAction
```

## UI组件

### SettingsPanel

大厅设置面板，集成所有设置选项的容器组件。

**属性：**
- `settings: LobbySettings` - 当前设置
- `isHost: boolean` - 是否为主机（可编辑）
- `onChange?: (settings: Partial<LobbySettings>) => void` - 设置变更回调
- `onSave?: () => void` - 保存设置回调
- `validationErrors?: Record<string, string>` - 验证错误
- `compact?: boolean` - 紧凑模式

### GameModeSelector

游戏模式选择器组件，用于选择和配置游戏模式。

**属性：**
- `currentMode: GameMode` - 当前选中的模式
- `onSelect: (mode: GameMode) => void` - 模式选择回调
- `disabled?: boolean` - 是否禁用
- `showDescription?: boolean` - 是否显示模式描述
- `highlightRecommended?: boolean` - 是否突出显示推荐模式

### CustomRulesForm

自定义规则配置表单，允许调整游戏规则设置。

**属性：**
- `rules: CustomRules` - 当前规则设置
- `onChange: (rules: Partial<CustomRules>) => void` - 规则变更回调
- `gameMode: GameMode` - 当前游戏模式
- `disabled?: boolean` - 是否禁用
- `errors?: Record<string, string>` - 验证错误
- `showAdvanced?: boolean` - 是否显示高级选项

### MapSelector

地图选择器组件，用于浏览和选择游戏地图。

**属性：**
- `maps: MapInfo[]` - 可用地图列表
- `selectedMapId: string` - 当前选中的地图ID
- `onSelect: (mapId: string) => void` - 地图选择回调
- `gameMode: GameMode` - 当前游戏模式
- `disabled?: boolean` - 是否禁用
- `showPreview?: boolean` - 是否显示地图预览

## 使用示例

### 设置面板集成

```tsx
import { useDispatch, useSelector } from 'react-redux';
import { 
  SettingsPanel,
  updateSettings,
  saveSettings,
  selectLobbySettings,
  validateSettings
} from '@features/current-lobby/lobby-settings';
import { selectIsHost } from '@features/current-lobby';

const LobbySettingsContainer = () => {
  const dispatch = useDispatch();
  const settings = useSelector(selectLobbySettings);
  const isHost = useSelector(selectIsHost);
  const [validationErrors, setValidationErrors] = useState({});
  
  // 处理设置变更
  const handleSettingsChange = (updatedSettings) => {
    // 验证设置
    const validation = validateSettings({
      ...settings,
      ...updatedSettings
    });
    
    setValidationErrors(validation.errors);
    
    if (validation.isValid) {
      dispatch(updateSettings(updatedSettings));
    }
  };
  
  // 保存设置
  const handleSaveSettings = () => {
    dispatch(saveSettings());
  };
  
  return (
    <div className="lobby-settings-container">
      <h2>大厅设置</h2>
      
      <SettingsPanel 
        settings={settings}
        isHost={isHost}
        onChange={handleSettingsChange}
        onSave={handleSaveSettings}
        validationErrors={validationErrors}
      />
      
      {!isHost && (
        <div className="settings-notice">
          只有主机可以修改设置
        </div>
      )}
    </div>
  );
};
```

### 游戏模式选择

```tsx
import { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { 
  GameModeSelector,
  setGameMode,
  loadDefaultSettings,
  selectCurrentGameMode
} from '@features/current-lobby/lobby-settings';

const GameModeSelection = () => {
  const dispatch = useDispatch();
  const currentMode = useSelector(selectCurrentGameMode);
  const [showConfirm, setShowConfirm] = useState(false);
  const [pendingMode, setPendingMode] = useState(null);
  
  // 处理模式选择
  const handleModeSelect = (mode) => {
    // 如果选择不同的模式，显示确认对话框
    if (mode !== currentMode) {
      setShowConfirm(true);
      setPendingMode(mode);
    }
  };
  
  // 确认模式变更
  const confirmModeChange = () => {
    // 加载该模式的默认设置
    dispatch(loadDefaultSettings(pendingMode));
    // 设置新模式
    dispatch(setGameMode(pendingMode));
    // 关闭确认对话框
    setShowConfirm(false);
  };
  
  return (
    <div className="game-mode-selection">
      <h3>游戏模式</h3>
      
      <GameModeSelector 
        currentMode={currentMode}
        onSelect={handleModeSelect}
        showDescription={true}
        highlightRecommended={true}
      />
      
      {/* 确认对话框 */}
      {showConfirm && (
        <ConfirmDialog
          title="更改游戏模式"
          message="更改游戏模式将重置当前设置。确定要继续吗？"
          onConfirm={confirmModeChange}
          onCancel={() => setShowConfirm(false)}
        />
      )}
    </div>
  );
};
```

### 自定义规则配置

```tsx
import { useDispatch, useSelector } from 'react-redux';
import { 
  CustomRulesForm,
  updateCustomRules,
  selectCustomRules,
  selectCurrentGameMode
} from '@features/current-lobby/lobby-settings';
import { selectIsHost } from '@features/current-lobby';

const CustomRulesConfig = () => {
  const dispatch = useDispatch();
  const customRules = useSelector(selectCustomRules);
  const gameMode = useSelector(selectCurrentGameMode);
  const isHost = useSelector(selectIsHost);
  const [showAdvanced, setShowAdvanced] = useState(false);
  
  // 处理规则变更
  const handleRulesChange = (updatedRules) => {
    dispatch(updateCustomRules(updatedRules));
  };
  
  // 切换显示高级选项
  const toggleAdvancedOptions = () => {
    setShowAdvanced(!showAdvanced);
  };
  
  // 在排位模式下禁用自定义规则
  const isDisabled = !isHost || gameMode === 'RANKED';
  
  return (
    <div className="custom-rules-config">
      <div className="section-header">
        <h3>游戏规则</h3>
        <button 
          onClick={toggleAdvancedOptions}
          className="advanced-toggle"
        >
          {showAdvanced ? '隐藏高级选项' : '显示高级选项'}
        </button>
      </div>
      
      {isDisabled && gameMode === 'RANKED' && (
        <div className="rules-notice">
          排位模式使用标准规则，无法自定义。
        </div>
      )}
      
      <CustomRulesForm 
        rules={customRules}
        onChange={handleRulesChange}
        gameMode={gameMode}
        disabled={isDisabled}
        showAdvanced={showAdvanced}
      />
    </div>
  );
};
```

## 与其他子模块的关系

- 使用**model**子模块的状态管理基础和操作接口
- 向父级**current-lobby**模块提供设置功能
- 与**lobby-interactions**子模块协作，在设置变更时通知玩家
- 配置影响**current-match**模块的游戏规则和环境
- 适配**maps**实体提供的地图数据

## 开发指南

1. 设置变更应立即反映在UI上，但应延迟同步到服务器以减少网络请求
2. 不同游戏模式的设置限制应清晰，自动禁用不适用的选项
3. 设置验证应全面且用户友好，提供明确的错误信息
4. 考虑设置预设功能，允许保存和加载常用设置组合
5. 为复杂设置提供工具提示和帮助信息，提高用户理解
6. 设置界面应响应式设计，适应不同屏幕尺寸
7. 设置变更应提供撤销/重做功能，方便用户试验
8. 对于重要设置变更，应请求确认以防止意外更改
9. 设置选项应分组和分类，提高可用性和可发现性 