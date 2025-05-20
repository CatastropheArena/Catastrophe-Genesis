"use client";

import React from 'react';
import { ConnectButton, Connector } from '@ant-design/web3';
import { useCurrentAccount } from '@mysten/dapp-kit';
import { usePassportData } from '../lib/passport/usePassportData';
import { styled } from '@mui/material';
import { getObjectUrl } from 'src/config/explorerList';

// 使用 Material-UI 的样式系统，更新为现代游戏风格
const StyledConnectButton = styled(ConnectButton)`
    /* 基础样式 */
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    font-family: "Miriam Libre", sans-serif;
    font-size: 1.4rem;
    font-weight: 700;
    text-transform: uppercase;
    min-width: 240px;
    padding: 0.8rem 1.6rem;
    border-width: 0 !important;
    border-radius: 8px;
    background: rgba(27, 32, 39, 0.9);
    color: #8BC34A;
    position: relative;
    cursor: pointer;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.25);
    overflow: hidden;
   
    
    @keyframes borderGlow {
        0% { background-position: 0% 0; }
        100% { background-position: 200% 0; }
    }
    

    
    /* 动画过渡 */
    transition: all 300ms cubic-bezier(0.34, 1.56, 0.64, 1);
    
    
    /* 激活状态 */
    &:active {
        transform: translateY(1px) scale(0.98);
        box-shadow: 0 2px 8px rgba(76, 175, 80, 0.2);
    }
    
    /* 禁用状态 */
    &:disabled {
        opacity: 0.65;
        cursor: not-allowed;
        background: rgba(20, 25, 30, 0.8);
        color: rgba(139, 195, 74, 0.5);
        transform: none;
        box-shadow: none;
    }

    /* 响应式设计 */
    @media (max-width: 640px) {
        font-size: 1.2rem;
        padding: 0.6rem 1.2rem;
    }

    /* 加载状态 */
    &[data-loading="true"] {
        cursor: wait;
        opacity: 0.8;
    }

    /* 图标样式 */
    svg {
        width: 1.8rem;
        height: 1.8rem;
        fill: currentColor;
        transition: transform 0.3s ease;
    }
    
    &:hover svg {
        transform: scale(1.15);
    }
    
    /* 覆盖网络选择和地址显示样式 */
    .ant-web3-connect-button-account {
        background: rgba(30, 35, 42, 0.9) !important;
        border-color: #4CAF50 !important;
        color: #8BC34A !important;
        border-radius: 6px !important;
        padding: 4px 8px !important;
        transition: all 0.3s ease !important;
        font-family: "Miriam Libre", monospace !important;
    }
    
    .ant-web3-connect-button-account-network {
        background: rgba(76, 175, 80, 0.1) !important;
        color: #AED581 !important;
        padding: 2px 6px !important;
        border-radius: 4px !important;
    }
    
    .ant-web3-connect-button-account-address {
        color: #DCEDC8 !important;
        background: rgba(0, 0, 0, 0.2) !important;
        padding: 2px 6px !important;
        border-radius: 4px !important;
        font-family: monospace !important;
    }
`;

const PassportConnectButton: React.FC = () => {
    const { passportData, loading } = usePassportData();
    const currentAccount = useCurrentAccount();

    return (
        <Connector>
            {loading ? (
                <StyledConnectButton
                    loading={true}
                    account={{
                        address: 'loading...',
                        name: 'loading...',
                    }}
                />
            ) : passportData && currentAccount ? (
                <StyledConnectButton
                    // avatar={{
                    //     src: passportData.display.image_url,
                    // }}
                    account={{
                        address: passportData.display.creator,
                        name: passportData.display.name,
                    }}
                    actionsMenu={{
                        extraItems: [{
                            key: '1',
                            label: '查看 Passport',
                            onClick: () => {
                                if (currentAccount && passportData?.objectId) {
                                    const accountUrl = getObjectUrl('testnet', passportData.objectId);
                                    window.open(accountUrl, '_blank')?.focus();
                                } else {
                                    alert("未连接钱包或未找到 Passport");
                                }
                            }
                        }]
                    }}
                />
            ) : (
                <StyledConnectButton />
            )}
        </Connector>
    );
};

export default PassportConnectButton; 