# 认证功能模块 (Auth Feature)

## 模块概述

认证模块是灾变创世录游戏平台的安全基础，负责用户身份验证、会话管理和访问控制。该模块处理用户登录、注册、权限验证和身份恢复等关键安全功能，确保只有授权用户才能访问受保护资源，为整个应用提供统一的身份上下文管理。

## 核心功能

- **身份认证**: 提供用户登录、注册和身份验证的完整流程，支持多种认证方式
- **会话管理**: 处理用户会话的创建、维护、刷新和销毁，确保安全的用户状态持久化
- **权限控制**: 提供基于角色和权限的访问控制系统，保护受限资源和路由
- **自动身份恢复**: 在页面刷新或会话中断后自动重新建立用户认证状态
- **认证状态管理**: 集中管理和同步应用中的用户认证状态
- **安全防护**: 实现防止CSRF、XSS等常见安全威胁的机制

## 关键组件

### 模型层 (model/)

- **store.ts**: 定义认证状态存储结构和更新逻辑，维护 `isAuthenticated` 和 `areCredentialsFetching` 等核心状态
- **actions.ts**: 提供认证相关的异步操作，包括 `fetchCredentials`、`signIn`、`signUp` 和 `verifyUsername` 等
- **selectors.ts**: 提供从全局状态中获取认证信息的选择器，如 `isAuthenticated` 和 `areCredentialsFetching`
- **index.ts**: 统一导出模型层组件，形成模型公共 API

### 工具组件 (lib/)

- **authenticated.tsx**: 条件渲染组件，根据用户认证状态显示不同内容，支持加载状态处理
- **credentials-obtainer.tsx**: 自动获取用户凭证的高阶组件，通常放置在应用顶层以实现自动登录
- **protected-route.tsx**: 保护路由组件，控制只有已认证用户才能访问特定页面

## 依赖关系

### 内部依赖

- **@entities/viewer**: 获取和管理当前用户信息
- **@app/store**: 访问全局 Redux 存储和 dispatch 工具
- **@shared/lib**: 使用公共工具函数如 Branch 组件
- **@shared/api**: 使用认证相关的 API 客户端

### 外部依赖

- **Redux Toolkit**: 状态管理和异步操作
- **Axios**: HTTP 请求处理和错误管理
- **React**: 组件构建与状态管理
- **MUI**: 使用 CircularProgress 等 UI 组件

## 使用示例

### 自动身份恢复

```tsx
// 在应用入口点使用 CredentialsObtainer
import { CredentialsObtainer } from '@features/auth';

const App = () => (
  <CredentialsObtainer>
    {/* 应用内容将在凭证获取后渲染 */}
    <MainContent />
  </CredentialsObtainer>
);
```

### 条件渲染认证内容

```tsx
// 根据认证状态显示不同内容
import { Authenticated } from '@features/auth';
import { useSelector } from 'react-redux';
import { authModel } from '@features/auth';

const Header = () => {
  // 获取认证状态
  const isAuthenticated = useSelector(authModel.selectors.isAuthenticated);
  
  return (
    <header>
      <Logo />
      <Authenticated
        render={({ credentials }) => (
          // 已登录用户看到的内容
          <UserMenu username={credentials.username} />
        )}
      >
        {/* 未登录用户看到的内容 */}
        <LoginButton />
      </Authenticated>
    </header>
  );
};
```

## 架构说明

认证模块采用了 Redux 状态管理模式，将状态、操作和选择器分离，形成清晰的数据流动：

1. 用户触发认证操作（如登录）
2. 异步 action 发送 API 请求，获取凭证
3. 成功后更新认证状态并存储凭证
4. 选择器暴露当前认证状态供组件使用
5. 工具组件根据认证状态条件渲染内容

模块通过高阶组件和条件渲染组件将认证逻辑从业务逻辑中分离，实现关注点分离，增强可维护性。

## 功能模块泳道流程图

```mermaid
sequenceDiagram
    participant User as 用户
    participant Auth as 认证模块
    participant API as 后端服务
    participant Viewer as 查看者实体
    participant Routes as 路由系统
    
    User->>Auth: 输入登录凭证
    Auth->>API: 发送登录请求
    API-->>Auth: 返回用户凭证
    Auth->>Auth: 更新认证状态
    Auth->>Viewer: 设置用户凭证
    Viewer->>API: 获取用户信息
    API-->>Viewer: 返回用户数据
    Viewer-->>Routes: 提供身份上下文
    Routes-->>User: 渲染授权页面
    
    Note over Auth,Viewer: 认证模块与查看者实体协作<br/>管理用户身份和访问控制
```

## 数据模型

```typescript
// 认证状态
export interface AuthState {
  isAuthenticated: boolean;
  areCredentialsFetching: boolean;
}

// 登录数据
export interface SignInData {
  username: string;
  password: string;
  rememberMe?: boolean;
}

// 注册数据
export interface SignUpData {
  username: string;
  email: string;
  password: string;
  confirmPassword: string;
}

// 认证操作结果
export interface AuthResponse {
  credentials: {
    id: string;
    token: string;
    username: string;
  }
}
```

## 最佳实践

1. 使用 `CredentialsObtainer` 在应用顶层自动恢复用户会话
2. 利用 `Authenticated` 组件实现基于认证状态的条件渲染
3. 保护敏感路由时使用 `ProtectedRoute` 组件
4. 监听认证状态变化以触发相应的UI更新和重定向
5. 确保所有异步认证操作都有适当的加载状态和错误处理
6. 在用户操作前验证认证状态，避免未授权访问 