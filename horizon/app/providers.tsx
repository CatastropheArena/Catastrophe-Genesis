"use client";

import { useEffect, useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { SuiClientProvider, WalletProvider } from "@mysten/dapp-kit";
import "@mysten/dapp-kit/dist/index.css";
import { networkConfig, network } from "@/contracts";
import { AppContextProvider } from "@/context/AppContext";
import SuiWalletProvider from "@/context/WalletContext";
import { TooltipProvider } from "@/components/ui/tooltip";
import { ToastContainer } from "react-toastify";
import { Inter } from "next/font/google";
import MetaTagsContainer from "@/components/metaTagsContainer/metaTagsContainer";
import { AssetsProvider } from "@/context/AssetsContext";
import { LoadingProvider } from "@/context/LoadingContext";
import { Loading } from "@/app/components/ui/loading";

const inter = Inter({ subsets: ["latin"] });

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient());
  const [isClient, setIsClient] = useState<boolean>(false);

  useEffect(() => {
    setIsClient(true);
  }, []);

  return (
    <>
      {isClient ? (
        <QueryClientProvider client={queryClient}>
          <SuiWalletProvider>
            <AppContextProvider>
              <TooltipProvider>
                <LoadingProvider>
                  <AssetsProvider>
                    {children}
                    <Loading />
                    <ToastContainer
                      theme="dark"
                      draggable
                      position="bottom-right"
                      className={"mt-20"}
                      toastClassName={(context) =>
                        "relative flex py-3.5 px-4 mx-4 min-h-10 mb-5 rounded-md justify-between overflow-hidden cursor-pointer bg-white/20 backdrop-blur-md"
                      }
                      toastStyle={{
                        WebkitBackdropFilter: "blur(12px)",
                      }}
                    />
                  </AssetsProvider>
                </LoadingProvider>
              </TooltipProvider>
            </AppContextProvider>
          </SuiWalletProvider>
        </QueryClientProvider>
      ) : (
        <div>
          <MetaTagsContainer />
        </div>
      )}
    </>
  );
}
