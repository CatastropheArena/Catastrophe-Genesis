import type { ReactNode } from "react"

interface FeatureCardProps {
  icon: ReactNode
  title: string
  description: string
}

export default function FeatureCard({ icon, title, description }: FeatureCardProps) {
  return (
    <div className="group bg-card border border-border/50 rounded-xl p-6 transition-all duration-300 hover:bg-card/95 hover:border-primary/50 hover:shadow-lg hover:shadow-primary/5">
      <div className="bg-primary/10 w-16 h-16 rounded-full flex items-center justify-center mb-4 text-primary transition-all duration-300 group-hover:scale-110 group-hover:bg-primary/15">
        {icon}
      </div>
      <h3 className="text-xl font-semibold mb-3 text-foreground">{title}</h3>
      <p className="text-muted-foreground/90 leading-relaxed">{description}</p>
    </div>
  )
}
