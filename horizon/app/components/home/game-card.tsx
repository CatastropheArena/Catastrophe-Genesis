interface GameCardProps {
    title: string
    description: string
    image: string
    color: string
    className?: string
  }
  
  export default function GameCard({ title, description, image, color, className = "" }: GameCardProps) {
    return (
      <div className={`w-64 h-96 rounded-xl overflow-hidden shadow-lg ${color} ${className}`}>
        <div className="p-4 h-full flex flex-col">
          <div className="text-white font-bold text-xl mb-2">{title}</div>
          <div className="bg-white/90 rounded-lg flex-1 flex items-center justify-center p-4">
            <img src={image || "/placeholder.svg"} alt={title} className="w-full h-auto" />
          </div>
          <div className="text-white/90 text-sm mt-4">{description}</div>
        </div>
      </div>
    )
  }
  