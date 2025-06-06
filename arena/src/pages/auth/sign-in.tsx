import React, { useState } from "react";
import { Link } from "react-router-dom";
import { useSnackbar } from "notistack";
import { useCurrentAccount, useSignPersonalMessage, useSuiClient } from "@mysten/dapp-kit";
import { Fullscreen } from "@shared/ui/templates";
import { Loader2, LogIn, Play } from "lucide-react";
import CustomConnectButton from "src/components/CustomConnectButton";
import { useDispatch } from "@app/store";
import { authModel } from "@features/auth";
import { viewerModel } from "@entities/viewer";
import { useNexusObjects } from "src/lib/nexus/usePassport";
import { SealApproveVerifyNexusPassportMoveCall, prepareSessionToken } from "src/lib/server/sessionToken";
import { getNetworkVariables } from "src/config/networkConfig";
import { useChainStore } from "src/components/chain";
import { useSessionStore } from 'src/components/session';
import {socket} from "@shared/lib/ws";
import { User } from "@entities/user";
import { useAuthStore } from 'src/components/auth';

export const SignInPage: React.FC = () => (
  <Fullscreen>
    {/* // 全屏容器 添加透明度*/}
    <div className="min-h-screen w-full flex items-center justify-center relative bg-white/10">
      {/* 背景渐变 - 优化light模式下的渐变效果 */}
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(139,195,74,0.15)_0%,rgba(104,58,183,0.25)_45%,rgba(76,175,80,0.35)_100%)] dark:bg-[radial-gradient(circle_at_center,rgba(104,58,183,0.05)_0%,rgba(76,175,80,0.15)_45%,rgba(156,39,176,0.25)_100%)] animate-gradient-shift pointer-events-none" />
      {/* 主面板 */}
      <div className="w-[90%] max-w-[500px] bg-[rgba(30,35,42,0.95)] backdrop-blur-2xl rounded-2xl p-9 border border-white/[0.06] shadow-[0_12px_40px_rgba(0,0,0,0.6)] relative overflow-hidden">
        {/* 标题区域 */}
        <div className="text-center mb-8">
          <h1 className="flex items-center justify-center gap-3 text-4xl font-black uppercase tracking-wide text-white font-bungee">
            <LogIn className="w-12 h-12 fill-[#8BC34A]" />
            Wallet Login
          </h1>
          <h2 className="text-lg font-medium uppercase tracking-wider text-white/70 font-miriam">
            Connect with your wallet to log in to the Explosion Cat game.
          </h2>
        </div>

        <SignInWithWallet />

        {/* 底部链接 */}
        <div className="w-full pt-8 mt-2 border-t border-white/10">
          <div className="flex items-center justify-center gap-4">
            <span className="text-lg uppercase tracking-wider text-white/60">
              No passport?
            </span>
            <Link
              to="/register"
              className="text-[#8BC34A] text-lg font-bold uppercase tracking-wider hover:text-[#AED581] hover:underline transition-colors"
            >
              Click here to create one.
            </Link>
          </div>
        </div>
      </div>
    </div>
  </Fullscreen>
);

const SignInWithWallet: React.FC = () => {
  const dispatch = useDispatch();
  const { enqueueSnackbar } = useSnackbar();
  const currentAccount = useCurrentAccount();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { mutate: signPersonalMessage } = useSignPersonalMessage();
  const { objects: objects, loading } = useNexusObjects();
  const currentChain = useChainStore((state) => state.currentChain);
  const networkConfig = getNetworkVariables(currentChain?.network || "testnet");
  const { getValidSession, renewSession } = useSessionStore();
  const { setToken } = useAuthStore();
  const suiClient = useSuiClient();

  const handleSessionKeyLogin = async () => {
    try {
      if (!currentAccount?.address) {
        throw new Error("Wallet address not obtained");
      }
      // 检查现有 session
      let sessionKey = getValidSession();
      // 如果没有有效 session，创建新的
      if (!sessionKey) {
        sessionKey = await renewSession(
          currentAccount.address,
          networkConfig?.CitadelPackage || "",
          signPersonalMessage
        );
        if (!sessionKey) {
          throw new Error("Failed to create session");
        }
      }

      // 准备 moveCall
      const moveCallConstructor = SealApproveVerifyNexusPassportMoveCall(
        networkConfig?.CitadelPackage || "",
        objects?.passport?.objectId || "",
        objects?.gameEntries[0]?.objectId || ""
      );

      // 获取 session token 数据
      const sessionTokenRequest = await prepareSessionToken(sessionKey, suiClient, moveCallConstructor);
      setIsSubmitting(true);
      console.log('开始登录流程...');
      dispatch(authModel.actions.signIn(sessionTokenRequest))
      .unwrap()
      .then((res) => {
        console.log('登录成功，响应数据:', res);
        // 保存 auth token
        setToken(res.auth_token);
        console.log('Token已保存');
        // 将 profile 数据转换为 User 格式
        const userData: User = {
          id: res.profile.id,
          username: currentAccount.address,
          avatar: res.profile.avatar,
          rating: res.profile.rating
        };
        console.log('准备设置用户数据:', userData);
        dispatch(
          viewerModel.actions.setCredentials({ credentials: userData })
        );
        console.log('用户数据已设置');
        socket.disconnect();
        socket.connect();
        console.log('Socket已重新连接');
      })
      .catch((error) => {
        console.error('登录失败:', error);
        enqueueSnackbar(error, {
          variant: "error",
        });
      })
      .finally(() => {
        setIsSubmitting(false);
      });
    } catch (error) {
      console.error("登录失败:", error);
      enqueueSnackbar("登录失败，请重试", { variant: "error" });
      setIsSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col items-center gap-6">
      {currentAccount ? (
        <>
          <CustomConnectButton
            onButtonClick={handleSessionKeyLogin}
            customNameComponent={
              isSubmitting ? (
                <div className="flex items-center">
                  <Loader2 className="w-8 h-8 animate-spin" />
                  LOGGING IN...
                </div>
              ) : (
                <div className="flex items-center gap-2 group">
                  <span className="font-bold tracking-wider">START GAME</span>
                  <Play className="w-5 h-5 text-white transition-transform duration-300 group-hover:translate-x-1" strokeWidth={2.5} />
                </div>
              )
            }
          />
          <div className="w-full border-t border-white/10 pt-4">
            <p className="text-lg uppercase tracking-wider text-white/60 text-center leading-relaxed">
              Click "START GAME" and confirm the signature request in your
              wallet to verify your identity
            </p>
          </div>
        </>
      ) : (
        <>
          <CustomConnectButton />
          <div className="w-full border-t border-white/10 pt-4">
            <p className="text-lg uppercase tracking-wider text-white/60 text-center leading-relaxed">
              Please connect your wallet to log in to the game
            </p>
          </div>
        </>
      )}
    </div>
  );
};
