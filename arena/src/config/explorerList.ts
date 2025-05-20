/**
 * 获取区块浏览器的基础URL
 * @param network 网络类型
 * @returns 区块浏览器基础URL
 */
export function getExplorerBaseUrl(network: Network): string {
  return `https://suiscan.xyz/${network}`;
}

/**
 * 获取交易详情页面URL
 * @param network 网络类型
 * @param txHash 交易哈希
 * @returns 交易详情页面URL
 */
export function getTransactionUrl(network: Network, txHash: string): string {
  return `${getExplorerBaseUrl(network)}/tx/${txHash}`;
}

/**
 * 获取对象详情页面URL
 * @param network 网络类型
 * @param objectId 对象ID
 * @returns 对象详情页面URL
 */
export function getObjectUrl(network: Network, objectId: string): string {
  return `${getExplorerBaseUrl(network)}/object/${objectId}`;
}

/**
 * 获取账户详情页面URL
 * @param network 网络类型
 * @param address 账户地址
 * @returns 账户详情页面URL
 */
export function getAccountUrl(network: Network, address: string): string {
  return `${getExplorerBaseUrl(network)}/account/${address}`;
}

// 备选的区块浏览器列表
export const explorerList = [
  {
    name: "Suiscan",
    getBaseUrl: getExplorerBaseUrl,
  },
  {
    name: "Sui Explorer",
    getBaseUrl: (network: Network) => 
      `https://explorer.sui.io/${network === 'mainnet' ? '' : network + '/'}`,
  }
]; 