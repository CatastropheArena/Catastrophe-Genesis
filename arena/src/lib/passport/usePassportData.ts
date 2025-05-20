import { useState, useEffect, useCallback } from 'react';
import { useCurrentAccount, useSuiClient } from '@mysten/dapp-kit';
import { SuiObjectData } from '@mysten/sui/client';
import { getNetworkVariables } from '../../config/networkConfig';
import { NETWORK } from '../../config/constants';
export interface PassportData {
    objectId: string;
    display: {
        name: string;
        creator: string;
        image_url: string;
        identify: string;
    };
}

export function usePassportData() {
    const [passportData, setPassportData] = useState<PassportData | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<Error | null>(null);
    const packageId = getNetworkVariables(NETWORK)?.NexusPackage;
    console.log(packageId);
    console.log(NETWORK);
    console.log(`${packageId}::passport::Passport`,);
    const currentAccount = useCurrentAccount();
    const client = useSuiClient();

    const fetchPassportData = useCallback(async () => {
        if (!currentAccount?.address || loading) return;

        setLoading(true);
        setError(null);

        try {
            const objects = await client.getOwnedObjects({
                owner: currentAccount.address,
                options: {
                    showContent: true,
                    showDisplay: true,
                },
                filter: {
                    MatchAny: [
                        {
                            StructType: `${packageId}::passport::Passport`,
                        },
                    ],
                },
            });

            if (objects.data.length > 0) {
                const data = objects.data[0].data as unknown as SuiObjectData;
                if (data.content?.dataType === 'moveObject') {
                    setPassportData({
                        objectId: data.objectId,
                        display: {
                            name: data.display?.data?.name || '未命名',
                            creator: currentAccount.address,
                            image_url: data.display?.data?.image_url || '',
                            identify: data.objectId,
                        },
                        // 可以根据实际的 Passport 结构添加更多字段
                        ...data.content.fields,
                    });
                }
            } else {
                setPassportData(null);
            }
        } catch (err) {
            console.error('获取 Passport 失败:', err);
            setError(err instanceof Error ? err : new Error('获取 Passport 时发生错误'));
        } finally {
            setLoading(false);
        }
    }, [currentAccount?.address, client]);

    useEffect(() => {
        if (currentAccount?.address) {
            fetchPassportData();
        } else {
            setPassportData(null);
        }
    }, [currentAccount?.address, fetchPassportData]);

    return {
        passportData,
        loading,
        error,
        refetch: fetchPassportData,
    };
} 