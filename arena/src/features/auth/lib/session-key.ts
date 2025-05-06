import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import { fromB64, toB64 } from '@mysten/sui/utils';
import { requestSigner } from '@shared/lib/wallet';

// SessionKey的默认有效期（分钟）
const DEFAULT_TTL_MIN = 10;

export interface SessionKeyCertificate {
  user: string;               // 用户钱包地址
  sessionKey: string;         // 会话公钥（base64编码）
  creationTime: number;       // 创建时间戳
  ttlMin: number;             // 有效期（分钟）
  signature: string;          // 用户签名
}

export class SessionKey {
  private address: string;
  private packageId: string;
  private creationTimeMs: number;
  private ttlMin: number;
  private sessionKey: Ed25519Keypair;
  private personalMessageSignature?: string;

  /**
   * 创建会话密钥实例
   */
  constructor({
    address,
    packageId,
    ttlMin = DEFAULT_TTL_MIN,
  }: {
    address: string;
    packageId: string;
    ttlMin?: number;
  }) {
    this.address = address;
    this.packageId = packageId;
    this.creationTimeMs = Date.now();
    this.ttlMin = ttlMin;
    this.sessionKey = Ed25519Keypair.generate();
  }

  /**
   * 检查会话密钥是否已过期
   */
  isExpired(): boolean {
    // 允许10秒的时钟偏差
    return this.creationTimeMs + this.ttlMin * 60 * 1000 - 10_000 < Date.now();
  }

  /**
   * 获取用户地址
   */
  getAddress(): string {
    return this.address;
  }

  /**
   * 获取包ID
   */
  getPackageId(): string {
    return this.packageId;
  }

  /**
   * 获取会话公钥（base64编码）
   */
  getSessionPublicKey(): string {
    return toB64(this.sessionKey.getPublicKey().toRawBytes());
  }

  /**
   * 获取会话创建时间
   */
  getCreationTime(): number {
    return this.creationTimeMs;
  }

  /**
   * 获取会话有效期（分钟）
   */
  getTtlMin(): number {
    return this.ttlMin;
  }

  /**
   * 获取需要用户签名的个人消息
   * 消息包含包ID、TTL、创建时间和会话公钥
   */
  getPersonalMessage(): Uint8Array {
    const creationTimeUtc =
      new Date(this.creationTimeMs).toISOString().slice(0, 19).replace('T', ' ') + ' UTC';
    const message = `Accessing Catastrophe Genesis with package ${this.packageId} for ${this.ttlMin} mins from ${creationTimeUtc}, session key ${this.getSessionPublicKey()}`;
    return new TextEncoder().encode(message);
  }

  /**
   * 获取需要用户签名的个人消息（字符串形式）
   */
  getPersonalMessageString(): string {
    return new TextDecoder().decode(this.getPersonalMessage());
  }

  /**
   * 设置个人消息的签名
   */
  setPersonalMessageSignature(personalMessageSignature: string) {
    this.personalMessageSignature = personalMessageSignature;
  }

  /**
   * 获取用户签名
   */
  getPersonalMessageSignature(): string | undefined {
    return this.personalMessageSignature;
  }

  /**
   * 获取证书对象，用于验证请求
   */
  getCertificate(): SessionKeyCertificate {
    if (!this.personalMessageSignature) {
      throw new Error('未设置个人消息签名');
    }
    
    return {
      user: this.address,
      sessionKey: this.getSessionPublicKey(),
      creationTime: this.creationTimeMs,
      ttlMin: this.ttlMin,
      signature: this.personalMessageSignature,
    };
  }

  /**
   * 请求用户签名
   * @returns 返回签名后的SessionKey
   */
  async requestSignature(): Promise<SessionKey> {
    if (this.personalMessageSignature) {
      return this;
    }

    try {
      const signResult = await requestSigner.signPersonalMessage({
        message: this.getPersonalMessage(),
      });
      
      this.setPersonalMessageSignature(signResult.signature);
      return this;
    } catch (error) {
      console.error('签名请求失败:', error);
      throw new Error('用户拒绝签名请求');
    }
  }
} 