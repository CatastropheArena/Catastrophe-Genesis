# Seal CLI 用户指南

Seal CLI是一个命令行工具，用于管理Seal密码系统中的密钥和加密操作。该工具提供了与Seal加密系统进行交互所需的全部功能，支持密钥生成、加密、解密和密钥派生等操作。

## 安装

Seal CLI作为Nautilus Server项目的一部分提供。您可以通过以下方式构建和安装：

```bash
# 克隆仓库
git clone https://github.com/CatastropheArena/Catastrophe-Genesis.git
cd Catastrophe-Genesis/citadel

# 构建项目
cargo build --release

# 工具将位于 target/release/seal-cli
```

## 使用方法

Seal CLI提供了多个子命令，每个子命令执行特定的功能。下面是各种命令的概述和使用示例。

### 生成密钥对

生成一个新的主密钥和公钥对：

```bash
seal-cli genkey
```

输出示例：
```
主密钥: 01e8889abc...
公钥: 0295e24d96...
```

### 提取用户私钥

从主密钥和用户ID提取用户私钥：

```bash
seal-cli extract --package-id 0x123abc... --id 75736572315f69645f737472696e67 --master-key 01e8889abc...
```

输出示例：
```
用户私钥: 03a5b7c9d0...
```

### 验证用户私钥

验证用户私钥是否与给定的公钥和ID匹配：

```bash
seal-cli verify --package-id 0x123abc... --id 75736572315f69645f737472696e67 --user-secret-key 03a5b7c9d0... --public-key 0295e24d96...
```

输出示例：
```
验证成功
```

### 使用明文模式派生密钥

派生一个密钥，使用明文模式（不加密任何消息）：

```bash
seal-cli plain --package-id 0x123abc... --id 75736572315f69645f737472696e67 --threshold 1 0295e24d96... 0x456def...
```

输出示例：
```
加密对象 (BCS编码): 01000000...
对称密钥: a1b2c3d4...
```

### 使用AES-256-GCM加密消息

使用AES-256-GCM加密模式加密消息：

```bash
seal-cli encrypt-aes --message 48656c6c6f20576f726c6421 --package-id 0x123abc... --id 75736572315f69645f737472696e67 --threshold 1 0295e24d96... 0x456def...
```

输出示例：
```
加密对象 (BCS编码): 01000000...
对称密钥: a1b2c3d4...
```

### 使用HMAC-256-CTR加密消息

使用HMAC-256-CTR加密模式加密消息：

```bash
seal-cli encrypt-hmac --message 48656c6c6f20576f726c6421 --package-id 0x123abc... --id 75736572315f69645f737472696e67 --threshold 1 0295e24d96... 0x456def...
```

输出示例：
```
加密对象 (BCS编码): 01000000...
对称密钥: a1b2c3d4...
```

### 解密Seal加密对象

使用私钥解密加密对象：

```bash
seal-cli decrypt 01000000... 03a5b7c9d0... 0x456def...
```

输出示例：
```
解密消息: 48656c6c6f20576f726c6421
```

### 使用对称密钥直接解密

当已知对称密钥时，直接解密加密对象：

```bash
seal-cli symmetric-decrypt 01000000... --key a1b2c3d4...
```

输出示例：
```
解密消息: 48656c6c6f20576f726c6421
```

### 解析Seal加密对象

查看加密对象的详细结构：

```bash
seal-cli parse 01000000...
```

输出示例：
```
版本: 1
包ID: 0x123abc...
ID: 75736572315f69645f737472696e67
服务器列表及共享索引:
  0x456def...: 0
阈值: 1
密文:
  类型: AES-256-GCM
  数据: 6a7b8c9d...
  额外认证数据: 无

加密份额:
  类型: Boneh-Franklin BLS12-381
  份额列表:
    e5f6g7h8...
  封装值: i9j0k1l2...
  加密随机性: m3n4o5p6...
```

## 自动化脚本示例

以下是一个自动化脚本示例，展示如何结合使用Seal CLI命令实现完整的加密和解密流程：

```bash
#!/bin/bash

# 生成密钥对
KEYS=$(seal-cli genkey)
MASTER_KEY=$(echo "$KEYS" | grep "主密钥" | cut -d' ' -f2)
PUBLIC_KEY=$(echo "$KEYS" | grep "公钥" | cut -d' ' -f2)

# 配置参数
PACKAGE_ID="0x123abc..."
USER_ID=$(echo -n "user_1" | xxd -p)

# 提取用户私钥
USER_SECRET_KEY=$(seal-cli extract --package-id $PACKAGE_ID --id $USER_ID --master-key $MASTER_KEY | cut -d' ' -f2)

# 加密消息
MESSAGE=$(echo -n "Hello, Seal!" | xxd -p)
ENCRYPTION=$(seal-cli encrypt-aes --message $MESSAGE --package-id $PACKAGE_ID --id $USER_ID --threshold 1 $PUBLIC_KEY $PACKAGE_ID)
ENCRYPTED_OBJECT=$(echo "$ENCRYPTION" | grep "加密对象" | cut -d' ' -f3)
SYM_KEY=$(echo "$ENCRYPTION" | grep "对称密钥" | cut -d' ' -f2)

# 解密消息 - 方法1：使用用户私钥
DECRYPTED1=$(seal-cli decrypt $ENCRYPTED_OBJECT $USER_SECRET_KEY $PACKAGE_ID)

# 解密消息 - 方法2：使用对称密钥
DECRYPTED2=$(seal-cli symmetric-decrypt $ENCRYPTED_OBJECT --key $SYM_KEY)

# 输出结果
echo "原始消息: $(echo -n $MESSAGE | xxd -r -p)"
echo "解密结果 (方法1): $(echo $DECRYPTED1 | grep '解密消息' | cut -d' ' -f2 | xxd -r -p)"
echo "解密结果 (方法2): $(echo $DECRYPTED2 | grep '解密消息' | cut -d' ' -f2 | xxd -r -p)"
```

## 错误处理

Seal CLI会在遇到错误时提供有用的错误消息并返回非零退出代码。最常见的错误包括：

- 无效的密钥格式
- 无效的加密对象
- 密钥数量不足（低于阈值）
- 用户私钥验证失败

如果遇到问题，请确保您提供的参数格式正确，并且所有必需的参数都已指定。

## 安全注意事项

使用Seal CLI时，请注意以下安全建议：

1. 主密钥应安全存储，不应暴露在不安全的环境中
2. 在生产环境中，避免将私钥或派生的对称密钥存储在明文文件中
3. 确保在安全的环境中运行加密和解密操作
4. 对于高安全性需求，考虑在隔离网络环境中使用Seal CLI 