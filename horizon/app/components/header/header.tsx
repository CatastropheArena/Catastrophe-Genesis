"use client";

import React, { useState, useContext, useEffect } from "react";
import { Coins, Gamepad2, Sparkles } from "lucide-react";
import { AppContext } from "@/context/AppContext";
import { ConnectModal } from "@mysten/dapp-kit";
import ConnectMenu from "@/app/components/ui/connectMenu";
import { Link as LinkIcon } from "lucide-react";
import { usePassport } from "@/hooks/usePassport";
import { useAssets } from "@/context/AssetsContext";
import { useNetwork } from "@/hooks/useNetwork";
import DialogModal from "@/app/components/dashboard/components/dialog-modal";

export const Header = () => {
  const [showWelcomeDialog, setShowWelcomeDialog] = useState(false);
  const [showNetworkDialog, setShowNetworkDialog] = useState(false);

  const { walletAddress, suiName } = useContext(AppContext);
  const {
    hasPassport,
    isCreating,
    isChecking,
    error: passportError,
    success,
    createNewUser,
  } = usePassport();
  const {
    assets,
    fetchAssets,
    isLoading: isLoadingAssets,
    error: assetsError,
  } = useAssets();
  const { isCorrectNetwork, expectedNetwork, checkNetwork } = useNetwork();

  // Monitor wallet connection, network, and passport status
  useEffect(() => {
    if (!walletAddress) {
      setShowNetworkDialog(false);
      setShowWelcomeDialog(false);
      return;
    }

    // Don't show any dialogs while checking passport status
    if (isChecking) {
      return;
    }

    // Check network status first
    if (isCorrectNetwork === false) {
      setShowNetworkDialog(true);
      setShowWelcomeDialog(false);
      return;
    }

    // Network is correct, check passport status
    if (isCorrectNetwork === true) {
      setShowNetworkDialog(false);

      // Show welcome dialog only if we've confirmed user doesn't have a passport
      if (hasPassport === false) {
        setShowWelcomeDialog(true);
      } else {
        setShowWelcomeDialog(false);
      }
    }
  }, [walletAddress, isCorrectNetwork, hasPassport, isChecking]);

  // Handle network change
  useEffect(() => {
    const handleNetworkChange = async () => {
      checkNetwork();
      // Refresh assets and check passport when network changes
      if (walletAddress) {
        await fetchAssets();
      }
    };

    window.addEventListener("sui_networkChange", handleNetworkChange);
    return () => {
      window.removeEventListener("sui_networkChange", handleNetworkChange);
    };
  }, [walletAddress, checkNetwork, fetchAssets]);

  // Handle new user creation
  const handleCreateUser = async () => {
    await createNewUser();
  };

  useEffect(() => {
    if (success && walletAddress) {
      console.log("Passport creation successful, refreshing assets...");
      const timer = setTimeout(async () => {
        await fetchAssets();
      }, 1000);
      return () => clearTimeout(timer);
    }
  }, [success, walletAddress, fetchAssets]);

  return (
    <>
      <header className="bg-purple-900/80 backdrop-blur-lg sticky top-0 z-40">
        <div className="container mx-auto px-2 py-2">
          <div className="flex justify-between items-center">
            <div className="flex items-center gap-2">
              <img
                src="/logo.png?height=120&width=120"
                alt="Logo"
                className="h-6 w-6"
              />
              <h1 className="text-base font-bold text-white hidden md:block">
                Catastrophe
              </h1>
            </div>

            <div className="flex items-center gap-2">
              {walletAddress ? (
                <>
                  <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
                    <Coins className="h-3 w-3 text-yellow-400" />
                    <span className="text-white font-medium text-xs">
                      {isLoadingAssets ? "..." : assets.coins}
                    </span>
                  </div>
                  <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
                    <Sparkles className="h-3 w-3 text-blue-400" />
                    <span className="text-white font-medium text-xs">
                      {isLoadingAssets ? "..." : assets.fragments}
                    </span>
                  </div>
                  <ConnectMenu
                    walletAddress={walletAddress}
                    suiName={suiName}
                  />
                </>
              ) : (
                <ConnectModal
                  trigger={
                    <div className="flex items-center gap-4">
                      <button
                        className="bg-primary hover:bg-primary/90 text-primary-foreground px-4 py-2 rounded-full flex items-center gap-2 transition-colors"
                        disabled={!!walletAddress}
                        >
                      {walletAddress ? "Connected" : "Connect Wallet"}
                    </button>
                  </div>
                  }
                />
              )}
            </div>
          </div>
        </div>
      </header>

      {/* Network Switch Dialog */}
      <DialogModal
        open={showNetworkDialog}
        onOpenChange={(open) => setShowNetworkDialog(open)}
        title="Network Switch Required"
        description={
          <>
            <p className="mb-4">
              Please switch your wallet network to{" "}
              <span className="font-bold text-purple-300">
                {expectedNetwork}
              </span>{" "}
              to continue.
            </p>
            <p className="text-sm text-purple-200">
              This game is currently only available on the {expectedNetwork}{" "}
              network. You'll need to switch networks in your wallet to play.
            </p>
          </>
        }
        type="network"
        confirmText="I Understand"
        data={null}
        isLoading={false}
        confirmAction={() => setShowNetworkDialog(false)}
        hideCancel={false}
        preventClose={false}
        hideCloseButton={false}
      />

      {/* Welcome Dialog */}
      <DialogModal
        open={showWelcomeDialog}
        onOpenChange={() => {}} // Prevent manual closing
        title="Welcome to Exploding Cats!"
        description={
          <>
            <p className="mb-4">
              To start your adventure in Exploding Cats, you'll need a Game
              Passport.
            </p>
            <p className="text-sm text-purple-200">
              This is a one-time process that will create your game profile and
              grant you initial rewards.
            </p>
          </>
        }
        type="welcome"
        confirmText={
          isCreating ? "Creating Passport..." : "Create Game Passport"
        }
        data={null}
        isLoading={isCreating}
        confirmAction={handleCreateUser}
        hideCancel={true}
        preventClose={true}
        hideCloseButton={true}
      />
    </>
  );
};
