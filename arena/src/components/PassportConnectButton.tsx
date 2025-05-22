"use client";

import React from 'react';
import { ConnectButton, Connector } from '@ant-design/web3';
import { useCurrentAccount } from '@mysten/dapp-kit';
import { usePassportData } from '../lib/passport/usePassportData';
import { getObjectUrl } from 'src/config/explorerList';

const buttonClasses='border-0 border-[#8BC34A] flex items-center justify-center px-6 py-3 bg-gradient-to-r from-[#4CAF50] via-[#8BC34A] to-[#4CAF50] bg-[length:200%_100%] text-white font-semibold text-lg uppercase tracking-wider shadow-lg hover:shadow-xl hover:scale-105 hover:brightness-110 transition-all duration-300 ease-in-out focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[#8BC34A]/50'
const PassportConnectButton: React.FC = () => {
    const { passportData, loading } = usePassportData();
    const currentAccount = useCurrentAccount();
    return (
        <Connector>
            {loading ? (
                <ConnectButton
                    className={buttonClasses}
                    loading={true}
                    account={{
                        address: 'loading...',
                        name: 'loading...',
                    }}
                />
            ) : passportData && currentAccount ? (
                <ConnectButton
                    className={buttonClasses}
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
                            label: 'View Passport',
                            onClick: () => {
                                if (currentAccount && passportData?.objectId) {
                                    const accountUrl = getObjectUrl('testnet', passportData.objectId);
                                    window.open(accountUrl, '_blank')?.focus();
                                } else {
                                    alert("No wallet connected or Passport not found");
                                }
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

export default PassportConnectButton; 