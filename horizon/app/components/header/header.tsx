"use client";

import React, { useState, useContext, useEffect } from "react";
import { Coins, Sparkles } from "lucide-react";
import { AppContext } from "@/context/AppContext";
import { ConnectModal, useDisconnectWallet } from "@mysten/dapp-kit";
import ConnectMenu from "@/app/components/ui/connectMenu";
import { Link as LinkIcon } from "lucide-react";
import { usePassport } from "@/hooks/usePassport";
import { useUserAssets } from "@/hooks/useUserAssets";
import { useNetwork } from "@/hooks/useNetwork";
import DialogModal from "@/app/components/dashboard/components/dialog-modal";

export default function Header() {
  const [showWelcomeDialog, setShowWelcomeDialog] = useState(false);
  const [showNetworkDialog, setShowNetworkDialog] = useState(false);

  const { walletAddress, suiName } = useContext(AppContext);
  const {
    hasPassport,
    isCreating,
    error: passportError,
    createNewUser,
  } = usePassport();
  const { assets, fetchAssets, error: assetsError } = useUserAssets();
  const { isCorrectNetwork, expectedNetwork } = useNetwork();
  const { mutate: disconnect } = useDisconnectWallet();

  // Monitor wallet connection, network, and passport status
  useEffect(() => {
    if (walletAddress) {
      if (isCorrectNetwork === false) {
        setShowNetworkDialog(true);
        setShowWelcomeDialog(false);
      } else if (isCorrectNetwork === true && hasPassport === false) {
        setShowNetworkDialog(false);
        setShowWelcomeDialog(true);
      }
    }
  }, [walletAddress, isCorrectNetwork, hasPassport]);

  // Handle new user creation
  const handleCreateUser = async () => {
    const success = await createNewUser();
    if (success) {
      setShowWelcomeDialog(false);
      // Refresh user assets
      await fetchAssets();
    }
  };

  return (
    <>
      <header className="bg-purple-900/80 backdrop-blur-lg sticky top-0 z-40">
        <div className="container mx-auto px-2 py-2">
          <div className="flex justify-between items-center">
            <div className="flex items-center gap-2">
              <img
                src="/placeholder.svg?height=40&width=40"
                alt="Logo"
                className="h-6 w-6"
              />
              <h1 className="text-base font-bold text-white hidden md:block">
                Exploding Cats
              </h1>
            </div>

            <div className="flex items-center gap-2">
              {walletAddress ? (
                <>
                  <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
                    <Coins className="h-3 w-3 text-yellow-400" />
                    <span className="text-white font-medium text-xs">
                      {assets.coins}
                    </span>
                  </div>
                  <div className="flex items-center gap-1 bg-black/30 px-2 py-1 rounded-full">
                    <Sparkles className="h-3 w-3 text-blue-400" />
                    <span className="text-white font-medium text-xs">
                      {assets.fragments}
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
                    <button
                      className="h-full rounded-[11px] outline-none ring-0 xl:button-animate-105 overflow-hidden p-[1px]"
                      disabled={!!walletAddress}
                    >
                      <div className="h-full px-5 py-4 flex items-center gap-2 rounded-xl bg-white/10">
                        <span className="text-sm">
                          {walletAddress ? "Connected" : "Connect Wallet"}
                        </span>
                        <LinkIcon size={17} className="text-white" />
                      </div>
                    </button>
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
        onOpenChange={() => {}} // Prevent manual closing
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
        confirmText=""
        data={null}
        isLoading={false}
        confirmAction={() => {}}
        hideCancel={true}
        preventClose={true}
        hideCloseButton={true}
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
}
