"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"
import { Wallet, ArrowRight } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"

export default function LoginPage() {
  const router = useRouter()
  const [isConnecting, setIsConnecting] = useState(false)
  const [walletAddress, setWalletAddress] = useState("")

  const connectWallet = async () => {
    setIsConnecting(true)

    // Simulate wallet connection
    setTimeout(() => {
      const mockAddress = "0x" + Math.random().toString(16).slice(2, 12) + "..."
      setWalletAddress(mockAddress)

      // Simulate successful connection and redirect
      setTimeout(() => {
        router.push("/dashboard")
      }, 1500)
    }, 1000)
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-purple-900 via-violet-800 to-indigo-900 flex items-center justify-center p-4">
      <Card className="w-full max-w-md bg-black/40 backdrop-blur-xl border-purple-500/30">
        <CardHeader className="text-center">
          <CardTitle className="text-3xl font-bold text-white">Connect Wallet</CardTitle>
          <CardDescription className="text-purple-200">
            Connect your wallet to start playing Exploding Cats
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex justify-center mb-6">
            <div className="w-24 h-24 rounded-full bg-gradient-to-br from-pink-500 to-purple-600 flex items-center justify-center">
              <Wallet className="h-12 w-12 text-white" />
            </div>
          </div>

          <div className="space-y-4">
            <Button
              onClick={connectWallet}
              disabled={isConnecting || !!walletAddress}
              className="w-full py-6 text-lg bg-gradient-to-r from-pink-600 to-purple-600 hover:from-pink-700 hover:to-purple-700 transition-all"
            >
              {isConnecting ? "Connecting..." : walletAddress ? "Connected to " + walletAddress : "Connect Metamask"}
            </Button>

            <Button
              variant="outline"
              className="w-full py-6 text-lg border-purple-500/50 text-purple-200 hover:bg-purple-900/30"
              onClick={connectWallet}
              disabled={isConnecting || !!walletAddress}
            >
              Connect WalletConnect
            </Button>
          </div>
        </CardContent>
        <CardFooter className="flex justify-center border-t border-purple-500/20 pt-4">
          {walletAddress && (
            <Button
              onClick={() => router.push("/dashboard")}
              className="bg-green-600 hover:bg-green-700 text-white flex items-center gap-2"
            >
              Continue to Game <ArrowRight className="h-4 w-4" />
            </Button>
          )}
        </CardFooter>
      </Card>
    </div>
  )
}

