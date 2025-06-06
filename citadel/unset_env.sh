#!/bin/bash

# 读取 .env 文件中的所有变量名并 unset
if [ -f .env ]; then
    echo "正在清除环境变量..."
    while IFS='=' read -r key value || [ -n "$key" ]; do
        # 跳过空行和注释
        [[ $key =~ ^[[:space:]]*# ]] && continue
        [[ -z "$key" ]] && continue
        
        # 提取变量名（去除前后空格和引号）
        key=$(echo "$key" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^["\x27]//' -e 's/["\x27]$//')
        
        # unset 变量
        unset "$key"
        echo "已清除: $key"
    done < .env
    echo "环境变量清除完成!"
else
    echo "错误: .env 文件不存在!"
    exit 1
fi

# 启动服务
echo "正在启动服务..."
cargo run 