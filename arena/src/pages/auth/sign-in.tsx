import React, { useEffect, useState } from "react";
import { styled } from "@mui/material";
import { Link, useNavigate } from "react-router-dom";
import { useSnackbar } from "notistack";

// import { useDispatch } from "@app/store";
// import { authModel } from "@features/auth";

import { Fullscreen, Center } from "@shared/ui/templates";
import { Layout } from "@shared/lib/layout";
import { H4, Text, Button, Loader } from "@shared/ui/atoms";
import { Icon } from "@shared/ui/icons";
import { viewerModel } from "@entities/viewer";
// import { SessionKey } from "@features/auth/lib/session-key";

export const SignInPage: React.FC = () => (
  <Fullscreen>
    <Center>
      <Layout.Col w={40} gap={2}>
        <Layout.Col gap={1}>
          <Layout.Row align="center" gap={1}>
            <WalletIcon />

            <H4>Wallet Login</H4>
          </Layout.Row>

          <Text
            emphasis="secondary"
            size={1.4}
            weight={700}
            transform="uppercase"
          >
            Connect with your wallet to log in to the Explosion Cat game.
          </Text>
        </Layout.Col>

        <SignInWithWallet />

        <Details>
          <Layout.Row align="center" gap={1}>
            <Help>No passport?</Help>
            <SignInLink to="/register">Click here to create one.</SignInLink>
          </Layout.Row>
        </Details>
      </Layout.Col>
    </Center>
  </Fullscreen>
);

const WalletIcon = styled(Icon.Login)`
  width: 4rem;
  fill: ${({theme}) => theme.palette.text.primary};
`;

const SignInWithWallet: React.FC = () => {
  const dispatch = useDispatch();
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [walletConnected, setWalletConnected] = useState(false);
  const [walletAddress, setWalletAddress] = useState<string | null>(null);

  // Connect wallet
  const handleConnectWallet = async () => {
    try {
      setIsSubmitting(true);
      await requestSigner.connect();
      const address = requestSigner.getAddress();
      
      if (!address) {
        throw new Error('Unable to get wallet address');
      }
      
      setWalletAddress(address);
      setWalletConnected(true);
      enqueueSnackbar('Wallet connected successfully', { variant: 'success' });
    } catch (error) {
      console.error('Wallet connection failed:', error);
      enqueueSnackbar('Failed to connect wallet, please try again', { variant: 'error' });
    } finally {
      setIsSubmitting(false);
    }
  };

  // Login with session key
  const handleSessionKeyLogin = async () => {
    try {
      setIsSubmitting(true);
      
      if (!walletAddress) {
        throw new Error('Wallet address not obtained');
      }
      
      // Create SessionKey
      const sessionKey = new SessionKey({
        address: walletAddress,
        packageId: PACKAGE_ID,
      });
      
      // Request user signature
      await sessionKey.requestSignature();
      
      // Get certificate
      const certificate = sessionKey.getCertificate();
      
      // Send to server for verification
      const result = await dispatch(
        authModel.actions.sessionKeyAuth({
          signature: certificate.signature,
          sessionKey: certificate.sessionKey,
          address: certificate.user,
          timestamp: certificate.creationTime,
          ttlMin: certificate.ttlMin,
        })
      ).unwrap();
      
      // Set user credentials
      dispatch(
        viewerModel.actions.setCredentials({ credentials: result.credentials })
      );
      
      enqueueSnackbar('Login successful', { variant: 'success' });
      
      // If no game entry, redirect to create passport page
      if (!result.hasGameEntry) {
        navigate('/create-passport');
      } else {
        navigate('/');
      }
      
    } catch (error) {
      console.error('Login failed:', error);
      enqueueSnackbar('Login failed, please try again', { variant: 'error' });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Layout.Col gap={2}>
      {!walletConnected ? (
        <Button
          variant="contained"
          startIcon={isSubmitting && <Spinner />}
          endIcon={!isSubmitting && <WalletConnectIcon />}
          onClick={handleConnectWallet}
          disabled={isSubmitting}
        >
          {isSubmitting ? "Connecting..." : "Connect Wallet"}
        </Button>
      ) : (
        <>
          <Layout.Col gap={1}>
            <Text emphasis="primary" size={1.2}>
              Wallet connected: {walletAddress?.slice(0, 6)}...{walletAddress?.slice(-4)}
            </Text>
            
            <Button
              variant="contained"
              startIcon={isSubmitting && <Spinner />}
              endIcon={!isSubmitting && <StartIcon />}
              onClick={handleSessionKeyLogin}
              disabled={isSubmitting}
            >
              {isSubmitting ? "Logging in..." : "Start Game"}
            </Button>
            
            <Text emphasis="secondary" size={1.2}>
              After clicking "Start Game", you need to confirm the signature request in your wallet to verify your identity
            </Text>
          </Layout.Col>
        </>
      )}
    </Layout.Col>
  );
};

const WalletConnectIcon = styled(Icon.Login)`
  width: 2rem;
  fill: ${({theme}) => theme.palette.primary.contrastText};
`;

const StartIcon = styled(Icon.Start)`
  width: 2rem;
  fill: ${({theme}) => theme.palette.primary.contrastText};
`;

const Spinner = styled(Loader.Spinner)`
  width: 2rem;
`;

const Details = styled(Layout.Col)`
  border-top: 1px solid ${({theme}) => theme.palette.divider};
  padding-top: 2rem;
`;

const Help = styled(Text)`
  color: ${({theme}) => theme.palette.text.secondary};
  font-size: 1.4rem;
  text-transform: uppercase;
`;

const SignInLink = styled(Link)`
  color: ${({theme}) => theme.palette.primary.main};
  font-size: 1.4rem;
  font-weight: 700;
  text-transform: uppercase;
`;
