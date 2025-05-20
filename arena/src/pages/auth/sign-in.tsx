import React, {  useState } from "react";
import { styled } from "@mui/material";
import { Link} from "react-router-dom";
import { useSnackbar } from "notistack";
import { useCurrentAccount } from '@mysten/dapp-kit';
import { SessionKey } from "@mysten/seal";
import PassportConnectButton from "../../components/PassportConnectButton";

// import { useDispatch } from "@app/store";
// import { authModel } from "@features/auth";
// import { viewerModel } from "@entities/viewer";

import { Fullscreen, Center } from "@shared/ui/templates";
import { Layout } from "@shared/lib/layout";
import { H4, Text, Button, Loader } from "@shared/ui/atoms";
import { Icon } from "@shared/ui/icons";
import { Box, Paper, Typography } from "@mui/material";
    
// 定义渐变按钮样式
const GradientButton = styled(Button)(`
  background: linear-gradient(90deg, #43A047 0%, #7CB342 100%);
  color: white;
  font-weight: 700;
  font-size: 1.4rem;
  padding: 12px 24px;
  border-radius: 8px;
  text-transform: uppercase;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  box-shadow: 0 6px 12px rgba(76, 175, 80, 0.25);
  position: relative;
  overflow: hidden;
  
  /* 按钮内光效 */
  &:before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 70%;
    height: 100%;
    background: linear-gradient(
      90deg, 
      transparent, 
      rgba(255, 255, 255, 0.2), 
      transparent
    );
    transition: 0.5s;
  }
  
  &:hover {
    transform: translateY(-3px) scale(1.02);
    box-shadow: 0 8px 20px rgba(76, 175, 80, 0.4);
    background: linear-gradient(90deg, #388E3C 0%, #689F38 100%);
    
    &:before {
      left: 100%;
    }
  }
  
  &:active {
    transform: translateY(1px) scale(0.98);
    box-shadow: 0 2px 8px rgba(76, 175, 80, 0.3);
  }
  
  /* 按钮上的图标 */
  & .MuiButton-startIcon,
  & .MuiButton-endIcon {
    transition: all 0.3s ease;
  }
  
  &:hover .MuiButton-startIcon,
  &:hover .MuiButton-endIcon {
    transform: scale(1.15);
  }
`);

// 定义游戏面板容器
const GamePanel = styled(Paper)(`
  background: rgba(18, 22, 28, 0.97);
  border-radius: 16px;
  padding: 36px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.06);
  backdrop-filter: blur(16px);
  max-width: 500px;
  width: 90%;
  position: relative;
  overflow: hidden;
  
  /* 背景发光效果 */
  &:before {
    content: '';
    position: absolute;
    top: -50%;
    left: -50%;
    width: 200%;
    height: 200%;
    background: radial-gradient(
      circle at center,
      rgba(76, 175, 80, 0.03) 0%,
      rgba(0, 0, 0, 0) 60%
    );
    z-index: -1;
  }
`);

// 定义地址显示样式
const AddressDisplay = styled(Box)(`
  background: rgba(30, 35, 42, 0.8);
  border-radius: 8px;
  padding: 10px 16px;
  font-family: monospace;
  color: #8BC34A;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 20px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  position: relative;
  overflow: hidden;
  
  /* 边框效果 */
  &:before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 8px;
    padding: 1px;
    background: linear-gradient(90deg, #4CAF50, #8BC34A, #4CAF50);
    background-size: 200% 100%;
    -webkit-mask: linear-gradient(#fff 0 0) content-box, 
                  linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    pointer-events: none;
    animation: borderGlow 3s infinite linear;
  }
  
  @keyframes borderGlow {
    0% { background-position: 0% 0; }
    100% { background-position: 200% 0; }
  }
`);

// 定义地址标识图标
const AddressIcon = styled(Icon.Login)(`
  width: 1.6rem;
  height: 1.6rem;
  margin-right: 8px;
  fill: #8BC34A;
`);

// 定义标题样式
const StyledTitle = styled(Typography)(`
  font-family: "Miriam Libre", sans-serif;
  font-size: 2.8rem;
  font-weight: 900;
  text-transform: uppercase;
  letter-spacing: 1px;
  margin: 0;
  color: white;
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
`);

// 定义子标题样式
const StyledSubtitle = styled(Typography)(`
  font-size: 1.4rem;
  font-weight: 500;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.7);
  letter-spacing: 0.5px;
  margin-bottom: 24px;
`);

const TTL_MIN = 10;
const PACKAGE_ID = import.meta.env.VITE_PACKAGE_ID || "";

export const SignInPage: React.FC = () => (
  <Fullscreen>
    <Box
      sx={{
        position: 'absolute',
        inset: 0,
        background: 'radial-gradient(circle at center, rgba(76, 175, 80, 0.03) 0%, rgba(0, 0, 0, 0) 70%)',
        pointerEvents: 'none'
      }}
    />
    <Center>

      <GamePanel>
        <Layout.Col align="center" gap={3}>
          <div style={{ textAlign: 'center' }}>
            <StyledTitle>
              <WalletIcon />
              Wallet Login
            </StyledTitle>
            <StyledSubtitle>
              Connect with your wallet to log in to the Explosion Cat game.
            </StyledSubtitle>
          </div>
          <div className="mx-auto flex max-w-sm items-center gap-x-4 rounded-xl bg-white p-6 shadow-lg outline outline-black/5 dark:bg-slate-800 dark:shadow-none dark:-outline-offset-1 dark:outline-white/10">
            <div>
              <div className="text-xl font-medium text-black dark:text-white">ChitChat</div>
              <p className="text-gray-500 dark:text-gray-400">You have a new message!</p>
            </div>
          </div>

   
          <SignInWithWallet />

          <Details>
            <Layout.Row align="center" gap={1}>
              <Help>No passport?</Help>
              <SignInLink to="/register">Click here to create one.</SignInLink>
            </Layout.Row>
          </Details>
        </Layout.Col>
      </GamePanel>
    </Center>
  </Fullscreen>
);



const SignInWithWallet: React.FC = () => {
  // const dispatch = useDispatch();
  // const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();
  const currentAccount = useCurrentAccount();
  // const { mutate: signPersonalMessage } = useSignPersonalMessage();

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
    <Layout.Col gap={3} align="center" sx={{ width: '100%' }}>
      {currentAccount ? (
        <>
          {/* 美化的钱包地址显示 */}
          <AddressDisplay>
            <AddressIcon />
            <Text emphasis="primary" size={1.2}>
              {currentAccount.address?.slice(0, 6)}...{currentAccount.address?.slice(-4)}
            </Text>
          </AddressDisplay>
          
          {/* 钱包连接按钮（支持退出登录） */}
          <PassportConnectButton />
          
          {/* 开始游戏按钮 */}
          <GradientButton
            startIcon={isSubmitting && <Spinner />}
            endIcon={!isSubmitting && <StartIcon />}
            onClick={handleSessionKeyLogin}
            disabled={isSubmitting}
          >
            {isSubmitting ? "LOGGING IN..." : "START GAME"}
          </GradientButton>
          
          <Text 
            emphasis="secondary" 
            size={1.1}
            sx={{ 
              textAlign: 'center', 
              maxWidth: '360px', 
              opacity: 0.7,
              color: '#e0e0e0' 
            }}
          >
            点击"START GAME"后，您需要在钱包中确认签名请求以验证身份
          </Text>
        </>
      ) : (
        <>
          <PassportConnectButton />
          
          <Text 
            emphasis="secondary" 
            size={1.1}
            sx={{ 
              textAlign: 'center', 
              opacity: 0.7,
              color: '#e0e0e0',
              maxWidth: '300px'
            }}
          >
            请先连接您的钱包以登录游戏
          </Text>
        </>
      )}
    </Layout.Col>
  );
};

const WalletIcon = styled(Icon.Login)`
  width: 3rem;
  height: 3rem;
  fill: #8BC34A;
`;

const StartIcon = styled(Icon.Start)`
  width: 2rem;
  height: 2rem;
  fill: white;
  transition: transform 0.3s ease;
`;

const Spinner = styled(Loader.Spinner)`
  width: 2rem;
  color: white;
  animation: spin 1s infinite linear;
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
`;

const Details = styled(Layout.Col)`
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding-top: 2rem;
  margin-top: 0.5rem;
  width: 100%;
`;

const Help = styled(Text)`
  color: rgba(255, 255, 255, 0.6);
  font-size: 1.3rem;
  text-transform: uppercase;
  letter-spacing: 0.5px;
`;

const SignInLink = styled(Link)`
  color: #8BC34A;
  font-size: 1.3rem;
  font-weight: 700;
  text-transform: uppercase;
  transition: color 0.2s ease;
  letter-spacing: 0.5px;
  &:hover {
    color: #AED581;
    text-decoration: underline;
  }
`;
