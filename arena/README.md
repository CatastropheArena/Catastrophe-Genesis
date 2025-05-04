# EKittens 项目文档

## 项目概述
EKittens是一个在线多人游戏平台，支持实时对战和社交功能。

### Web 客户端 (@web)

位置：`/apps/web`

#### 技术栈
- React 18
- TypeScript
- Vite
- Redux Toolkit
- Material-UI (MUI)
- React Router
- Socket.IO Client
- i18next (国际化)

#### 主要功能模块
- `/src/app` - 应用核心配置
- `/src/features` - 业务功能模块
- `/src/entities` - 业务实体
- `/src/pages` - 页面组件
- `/src/shared` - 共享工具和组件
- `/src/widgets` - 可复用的UI组件


#### 核心功能模块

1. 用户系统 (`/src/features/auth/`)
   - 用户认证与授权
     - `lib/authenticated.ts` - 认证状态组件
     - `lib/credentials-obtainer.ts` - 凭证获取
     - `model/` - 用户认证状态管理
   - 用户偏好设置 (`/src/features/preferences/`)
   - 个人资料管理

2. 游戏大厅系统
   - 创建游戏大厅 (`/src/features/create-lobby/`)
   - 加入现有大厅 (`/src/features/join-lobby/`)
   - 当前大厅状态 (`/src/features/current-lobby/`)
   - 大厅重连功能 (`/src/features/lobby-rejoin/`)
   - 大厅聊天功能 (通过 `/src/features/chat/` 实现)

3. 匹配系统
   - 快速匹配队列 (`/src/features/matchmaking-queue/`)
   - 加入指定比赛 (`/src/features/join-match/`)
   - 比赛重连功能 (`/src/features/match-rejoin/`)
   - 当前比赛状态 (`/src/features/current-match/`)

4. 对战系统 (`/src/features/current-match/`)
   - 实时游戏对战
     - `model/` - 游戏状态管理
     - `ui/` - 游戏界面组件
   - 当前比赛状态显示
   - 游戏操作控制
   - 比赛结果展示

5. 社交系统 (`/src/features/chat/`)
   - 实时聊天功能
     - `model/` - 聊天状态管理
     - `ui/` - 聊天界面组件
   - 好友系统
   - 社交互动
