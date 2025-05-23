import { Wallet, Gift, Gamepad2, Trophy } from "lucide-react"

export default function UserJourney() {
  const steps = [
    {
      icon: <Wallet className="h-8 w-8" />,
      title: "Connect Sui Wallet",
      description: "Connect your Sui wallet, create your player profile, and start your GameFi journey",
    },
    {
      icon: <Gift className="h-8 w-8" />,
      title: "Claim Starter Deck",
      description: "Receive your starter deck with basic cards and $CATA tokens to begin your collection",
    },
    {
      icon: <Gamepad2 className="h-8 w-8" />,
      title: "Battle & Earn",
      description:
        "Compete in strategic card battles, earn rewards, and collect Genesis Fragments to enhance your deck",
    },
    {
      icon: <Trophy className="h-8 w-8" />,
      title: "Evolve & Compete",
      description:
        "Synthesize powerful cards, join tournaments, stake assets for passive income, and climb the leaderboards",
    },
  ]

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
      {steps.map((step, index) => (
        <div key={index} className="relative group">
          <div className="bg-card border border-border/50 rounded-xl p-6 h-full transition-all duration-300 hover:bg-card/95 hover:border-primary/50 hover:shadow-lg hover:shadow-primary/5">
            <div className="bg-primary/10 w-16 h-16 rounded-full flex items-center justify-center mb-4 text-primary transition-all duration-300 group-hover:scale-110 group-hover:bg-primary/15">
              {step.icon}
            </div>
            <h3 className="text-xl font-semibold mb-3 text-foreground">{step.title}</h3>
            <p className="text-muted-foreground/90 leading-relaxed">{step.description}</p>
          </div>

          {index < steps.length - 1 && (
            <div className="hidden lg:block absolute top-1/2 -right-4 transform -translate-y-1/2 z-10">
              <div className="w-8 h-8 rounded-full bg-primary flex items-center justify-center transition-all duration-300 group-hover:scale-110 group-hover:bg-primary/90">
                <ArrowRight className="h-4 w-4 text-primary-foreground" />
              </div>
            </div>
          )}
        </div>
      ))}
    </div>
  )
}

import { ArrowRight } from "lucide-react"
