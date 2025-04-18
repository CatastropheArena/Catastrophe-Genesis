import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function HomePage() {
  return (
    <div className="relative min-h-screen bg-gradient-to-b from-purple-900 via-violet-800 to-indigo-900 overflow-hidden">
      {/* Background particles */}
      <div className="absolute inset-0 overflow-hidden">
        {Array.from({ length: 20 }).map((_, i) => (
          <div
            key={i}
            className="absolute rounded-full bg-white/10 animate-float"
            style={{
              width: `${Math.random() * 10 + 5}px`,
              height: `${Math.random() * 10 + 5}px`,
              top: `${Math.random() * 100}%`,
              left: `${Math.random() * 100}%`,
              animationDuration: `${Math.random() * 10 + 10}s`,
              animationDelay: `${Math.random() * 5}s`,
            }}
          />
        ))}
      </div>

      <div className="container mx-auto px-4 py-12 flex flex-col items-center justify-center min-h-screen relative z-10">
        <div className="w-full max-w-6xl mx-auto text-center">
          <div className="mb-8 animate-bounce-slow">
            <img
              src="/placeholder.svg?height=150&width=150"
              alt="Exploding Cats Logo"
              className="mx-auto h-32 w-32 md:h-40 md:w-40"
            />
          </div>

          <h1 className="text-4xl md:text-6xl font-bold mb-4 text-transparent bg-clip-text bg-gradient-to-r from-pink-500 via-red-500 to-yellow-500">
            Exploding Cats
          </h1>
          <p className="text-xl md:text-2xl mb-8 text-purple-100">
            The ultimate GameFi card battle experience
          </p>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-2xl mx-auto mb-12">
            <div className="bg-white/10 backdrop-blur-md rounded-xl p-6 text-left">
              <h3 className="text-xl font-semibold mb-2 text-pink-300">
                Strategic Gameplay
              </h3>
              <p className="text-purple-100">
                Destroy cards to gain resources and activate your deck to
                increase your win rate.
              </p>
            </div>
            <div className="bg-white/10 backdrop-blur-md rounded-xl p-6 text-left">
              <h3 className="text-xl font-semibold mb-2 text-pink-300">
                Earn & Collect
              </h3>
              <p className="text-purple-100">
                Win matches to earn resources and synthesize powerful cards for
                your collection.
              </p>
            </div>
          </div>

          <Link href="/login" className="inline-block">
            <Button
              size="lg"
              className="bg-gradient-to-r from-pink-600 to-purple-600 hover:from-pink-700 hover:to-purple-700 text-white font-bold py-3 px-8 rounded-full text-lg transition-all duration-300 hover:scale-105 shadow-lg"
            >
              Connect Wallet to Play
            </Button>
          </Link>
        </div>
      </div>
    </div>
  );
}
