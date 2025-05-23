import { useState, useEffect, useCallback } from 'react';
import { useCurrentAccount, useSuiClient } from '@mysten/dapp-kit';
import { SuiObjectData } from '@mysten/sui/client';
import { getNetworkVariables } from '../../config/networkConfig';
import { NETWORK } from '../../config/constants';

// 定义对象类型接口
export interface PassportData {
    objectId: string;
}
export interface GameEntryData {
    objectId: string;
}

// 定义返回数据接口
export interface NexusObjects {
    passport: PassportData | null;
    gameEntries: GameEntryData[];
}

export function useNexusObjects() {
    const [objects, setObjects] = useState<NexusObjects>({
        passport: null,
        gameEntries: [],
    });
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<Error | null>(null);
    const packageId = getNetworkVariables(NETWORK)?.NexusPackage;
    const currentAccount = useCurrentAccount();
    const client = useSuiClient();

    const fetchObjects = useCallback(async () => {
        if (!currentAccount?.address || loading) return;

        setLoading(true);
        setError(null);

        try {
            console.log("packageId", packageId);
            console.log("currentAccount", currentAccount.address);
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
                        {
                            StructType: `${packageId}::game::GameEntry`,
                        },
                    ],
                },
            });
            console.log("objects", objects);
            // 初始化结果对象
            const result: NexusObjects = {
                passport: null,
                gameEntries: [],
            };

            // 处理获取到的对象
            objects.data.forEach((obj) => {
                const data = obj.data as unknown as SuiObjectData;
                if (data.content?.dataType === 'moveObject') {
                    const type = data.content.type;
                    if (type.includes('::passport::Passport')) {
                        result.passport = {
                            objectId: data.objectId,
                            ...data.content.fields,
                        };
                    } else if (type.includes('::game::GameEntry')) {
                        result.gameEntries.push({
                            objectId: data.objectId,
                            ...data.content.fields,
                        });
                    }
                }
            });

            setObjects(result);
        } catch (err) {
            console.error('获取对象失败:', err);
            setError(err instanceof Error ? err : new Error('获取对象时发生错误'));
        } finally {
            setLoading(false);
        }
    }, [currentAccount?.address, client, packageId]);

    useEffect(() => {
        if (currentAccount?.address) {
            fetchObjects();
        } else {
            setObjects({ passport: null, gameEntries: [] });
        }
    }, [currentAccount?.address, fetchObjects]);

    return {
        objects,
        loading,
        error,
        refetch: fetchObjects,
    };
} 