import React, { useState } from "react";
import { Link } from "react-router-dom";
import { useSnackbar } from "notistack";
import { useCurrentAccount } from '@mysten/dapp-kit';
import { SessionKey } from "@mysten/seal";
import PassportConnectButton from "../../components/PassportConnectButton";
import { Fullscreen } from "@shared/ui/templates";
import { Loader2,LogIn,Play } from "lucide-react";

const TTL_MIN = 10;
const PACKAGE_ID = import.meta.env.VITE_PACKAGE_ID || "";

export const SignInPage: React.FC = () => (
  <Fullscreen>
  {/* // 全屏容器 添加透明度*/}
  <div className="min-h-screen w-full flex items-center justify-center relative dark:bg-slate-900/10 ">
    {/* 背景渐变 */}
    <div className="absolute inset-0 dark:bg-[radial-gradient(circle_at_center,rgba(104,58,183,0.05)_0%,rgba(76,175,80,0.15)_45%,rgba(156,39,176,0.25)_100%)] light:bg-[radial-gradient(circle_at_center,rgba(139,195,74,0.35)_0%,rgba(104,58,183,0.45)_45%,rgba(76,175,80,0.55)_100%)] animate-gradient-shift pointer-events-none" />
    {/* 主面板 */}
    <div className="w-[90%] max-w-[500px] bg-[rgba(30,35,42,0.8)] dark:bg-[rgba(18,22,28,0.97)] backdrop-blur-2xl rounded-2xl p-9 border border-white/[0.06] shadow-[0_12px_40px_rgba(0,0,0,0.6)] relative overflow-hidden">
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
  const { enqueueSnackbar } = useSnackbar();
  const currentAccount = useCurrentAccount();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSessionKeyLogin = async () => {
    try {
      setIsSubmitting(true);
      
      if (!currentAccount?.address) {
        throw new Error('Wallet address not obtained');
      }
      
      // Create SessionKey
      const sessionKey = new SessionKey({
        address: currentAccount.address,
        packageId: PACKAGE_ID,
        ttlMin: TTL_MIN,
      });
      console.log(sessionKey);

      // // Request user signature
      // signPersonalMessage(
      //   {
      //     message: sessionKey.getPersonalMessage(),
      //   },
      //   {
      //     onSuccess: async (signResult) => {
      //       await sessionKey.setPersonalMessageSignature(signResult.signature);
      //       const certificate = await sessionKey.getCertificate();
            
      //       // Send to server for verification
      //       const authResult = await dispatch(
      //         authModel.actions.sessionKeyAuth({
      //           signature: certificate.signature,
      //           sessionKey: certificate.session_vk,
      //           address: certificate.user,
      //           timestamp: certificate.creation_time,
      //           ttlMin: certificate.ttl_min,
      //         })
      //       ).unwrap();
            
      //       // Set user credentials
      //       if (authResult.credentials) {
      //         dispatch(
      //           viewerModel.actions.setCredentials({ credentials: authResult.credentials })
      //         );
              
      //         enqueueSnackbar('Login successful', { variant: 'success' });
              
      //         // If no game entry, redirect to create passport page
      //         if (authResult.hasGameEntry === false) {
      //           navigate('/create-passport');
      //         } else {
      //           navigate('/');
      //         }
      //       } else {
      //         throw new Error('No credentials returned from server');
      //       }
      //     },
      //   }
      // );
    } catch (error) {
      console.error('Login failed:', error);
      enqueueSnackbar('Login failed, please try again', { variant: 'error' });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col items-center gap-6">
      {currentAccount ? (
        <>
          <PassportConnectButton />

          {/* 钱包地址显示 */}
          <button className="group border-0 border-[#8BC34A] flex items-center justify-center px-6 py-3 rounded-lg bg-gradient-to-r from-[#4CAF50] via-[#8BC34A] to-[#4CAF50] bg-[length:200%_100%] animate-borderGlow text-white font-semibold text-lg uppercase tracking-wider shadow-lg hover:shadow-xl hover:scale-105 hover:brightness-110 transition-all duration-300 ease-in-out focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[#8BC34A]/50">
            {/* 发光边框动画 */}
            {isSubmitting ? (
              <>
                <Loader2 className="w-8 h-8 animate-spin" />
                LOGGING IN...
              </>
            ) : (
              <>
                START GAME
                <Play className="ml-4 w-8 h-8 text-white transition-transform group-hover:scale-110" />
              </>
            )}
          </button>

          <div className="w-full border-t border-white/10 pt-4">
            <p className="text-lg uppercase tracking-wider text-white/60 text-center leading-relaxed">
              Click "START GAME" and confirm the signature request in your wallet to verify your identity
            </p>
          </div>
        </>
      ) : (
        <>
          <PassportConnectButton />
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