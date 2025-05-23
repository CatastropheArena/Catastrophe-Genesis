// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * 密钥获取模块
 * 提供从密钥服务器获取和处理密钥的功能
 */

import { fromBase64, fromHex, toBase64, toHex } from '@mysten/bcs';
import { SealClient, SessionKey, NoAccessError } from '@mysten/seal';
import { SuiClient } from '@mysten/sui/client';
import { Transaction } from '@mysten/sui/transactions';
import { authApi, Certificate, SessionTokenRequest, SessionTokenResponse } from '@shared/api/auth';
import { elgamalDecrypt, toPublicKey, toVerificationKey } from './elgamal';

/**
 * 构造Move调用参数
 * 
 * 创建用于许可名单访问验证的交易调用
 * 
 * @param packageId - 包ID
 * @param allowlistId - 许可名单ID
 * @returns 交易构造函数
 */
export function SealApproveVerifyNexusPassportMoveCall(packageId: string,passportId: string, gameEntryId: string): SessionTokenMoveCallConstructor {
    return (tx: Transaction) => {
        tx.moveCall({
          target: `${packageId}::citadel::seal_approve_verify_nexus_passport`,
          arguments: [tx.pure.vector('u8', fromHex(passportId)),tx.object(passportId), tx.object(gameEntryId)],
        });
    };
}
export type SessionTokenMoveCallConstructor = (tx: Transaction) => void;

export const prepareSessionToken = async (
  sessionKey: SessionKey,
  suiClient: SuiClient,
  moveCallConstructor: SessionTokenMoveCallConstructor
): Promise<SessionTokenRequest> => {
    try {
      const tx = new Transaction();
      moveCallConstructor(tx);
      const txBytes = await tx.build({ client: suiClient,onlyTransactionKind: true });
      const cert = await sessionKey.getCertificate();
      const signedRequest = await sessionKey.createRequestParams(txBytes);
      // 从私钥生成公钥和验证密钥
      const encKeyPk = toPublicKey(signedRequest.decryptionKey);
      const encVerificationKey = toVerificationKey(signedRequest.decryptionKey);
      // 构建请求体
      const request: SessionTokenRequest = {
        ptb: toBase64(txBytes.slice(1)),
        enc_key: toBase64(encKeyPk),
        enc_verification_key: toBase64(encVerificationKey),
        request_signature: signedRequest.requestSignature,
        certificate:cert,
      };
      return request;
    } catch (err) {
      const errorMsg =
        err instanceof NoAccessError
          ? 'No access to decryption keys'
          : 'Unable prepare session token, try again';
      console.error(errorMsg, err);
      throw errorMsg;
    }
};