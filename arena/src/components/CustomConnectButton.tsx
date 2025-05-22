"use client";
import React from 'react';
import { ConnectButton, Connector } from '@ant-design/web3';
import { useCurrentAccount } from '@mysten/dapp-kit';
import { getObjectUrl } from 'src/config/explorerList';
import { useResolveSuiNSName } from "@mysten/dapp-kit";
import { usePassportData } from '../lib/passport/usePassportData';

const buttonClasses='border-0 hover:border-[#8BC34A] border-[#8BC34A] flex items-center justify-center px-6 py-3 bg-gradient-to-r from-[#4CAF50] via-[#8BC34A] to-[#4CAF50] bg-[length:200%_100%] text-white font-semibold text-lg uppercase tracking-wider shadow-lg hover:shadow-xl hover:scale-105 hover:brightness-110 transition-all duration-300 ease-in-out focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[#8BC34A]/50'

interface PassportConnectButtonProps {
    customNameComponent?: React.ReactNode;
    onButtonClick?: (e: React.MouseEvent) => void;
}

const CustomConnectButton: React.FC<PassportConnectButtonProps> = ({ 
    customNameComponent,
    onButtonClick 
}) => {
    const { passportData, loading } = usePassportData();
    const currentAccount = useCurrentAccount();
    const { data: suiName } = useResolveSuiNSName(currentAccount?.address);
    const defaultNameComponent = (currentAccount?.address);
    return (
        <Connector>
            {loading ? (
                <ConnectButton
                    className={buttonClasses}
                    loading={true}
                    account={{
                        address: 'Loading...',
                        name: 'Loading...',
                    }}
                />
            ) : currentAccount ? (
                <ConnectButton
                    className={buttonClasses}
                    profileModal={!onButtonClick}
                    onClick={onButtonClick ? (e) => {
                        onButtonClick(e);
                    } : undefined}
                    account={{
                        address: currentAccount.address,
                        // @ts-ignore
                        name: customNameComponent || defaultNameComponent,
                    }}
                    actionsMenu={{
                        extraItems: [{
                            key: '1',
                            label: suiName || `${currentAccount?.address.slice(0, 6)}...${currentAccount?.address.slice(-4)}`,
                            onClick: () => {
                                if (currentAccount) {
                                    navigator.clipboard.writeText(currentAccount.address)
                                        .then(() => {
                                            alert('Address copied to clipboard!');
                                        })
                                        .catch(err => {
                                            console.error('Failed to copy address:', err);
                                            alert('Failed to copy address');
                                        });
                                } else {
                                    alert("No wallet connected");
                                }
                            }
                        },
                        passportData && currentAccount ? {
                            key: '3',
                            label: 'View Passport',
                            onClick: () => {
                                if (currentAccount && passportData?.objectId) {
                                    const accountUrl = getObjectUrl('testnet', passportData.objectId);
                                    window.open(accountUrl, '_blank')?.focus();
                                } else {
                                    alert("No wallet connected or Passport not found");
                                }
                            }
                        } : {
                            key: '3',
                            label: 'Get Passport',
                            onClick: () => {
                                alert("No wallet connected or Passport not found");
                            }
                        }]
                    }}
                />
            ) : (
                <ConnectButton
                    className={buttonClasses}
                />
            )}
        </Connector>
    );
};

export default CustomConnectButton; 