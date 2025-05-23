"use client"

import { useEffect, useRef } from "react"

export default function TokenDistribution() {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext("2d")
    if (!ctx) return

    // 设置画布尺寸
    const setCanvasSize = () => {
      const container = canvas.parentElement
      if (container) {
        canvas.width = container.clientWidth
        canvas.height = Math.min(container.clientWidth * 0.6, 400)
      }
    }

    setCanvasSize()
    window.addEventListener("resize", setCanvasSize)

    // 绘制饼图
    const drawPieChart = () => {
      const width = canvas.width
      const height = canvas.height
      const centerX = width / 2
      const centerY = height / 2
      const radius = Math.min(width, height) / 2.5

      // 清除画布
      ctx.clearRect(0, 0, width, height)

      // 定义数据
      const data = [
        { label: "Initial Circulation", value: 10, color: "#9333ea" },
        { label: "Liquidity", value: 10, color: "#a855f7" },
        { label: "Match Rewards", value: 30, color: "#c084fc" },
        { label: "Card Staking", value: 5, color: "#d8b4fe" },
        { label: "Community Incentives", value: 10, color: "#e9d5ff" },
        { label: "Dev Incentives", value: 10, color: "#f3e8ff" },
        { label: "Card Development", value: 5, color: "#faf5ff" },
        { label: "Reserved", value: 20, color: "#7e22ce" },
      ]

      // 计算总和
      const total = data.reduce((sum, item) => sum + item.value, 0)

      // 绘制饼图
      let startAngle = 0

      data.forEach((item) => {
        const sliceAngle = (item.value / total) * 2 * Math.PI

        ctx.beginPath()
        ctx.moveTo(centerX, centerY)
        ctx.arc(centerX, centerY, radius, startAngle, startAngle + sliceAngle)
        ctx.closePath()

        ctx.fillStyle = item.color
        ctx.fill()

        // 绘制标签线和文本
        const midAngle = startAngle + sliceAngle / 2
        const labelRadius = radius * 1.2
        const labelX = centerX + Math.cos(midAngle) * labelRadius
        const labelY = centerY + Math.sin(midAngle) * labelRadius

        // 绘制连接线
        ctx.beginPath()
        ctx.moveTo(centerX + Math.cos(midAngle) * radius, centerY + Math.sin(midAngle) * radius)
        ctx.lineTo(labelX, labelY)
        ctx.strokeStyle = item.color
        ctx.lineWidth = 2
        ctx.stroke()

        // 绘制文本
        ctx.font = "12px sans-serif"
        ctx.fillStyle = "#64748b"
        ctx.textAlign = midAngle < Math.PI ? "left" : "right"
        ctx.textBaseline = "middle"
        ctx.fillText(`${item.label} ${item.value}%`, midAngle < Math.PI ? labelX + 5 : labelX - 5, labelY)

        startAngle += sliceAngle
      })
    }

    drawPieChart()
    window.addEventListener("resize", drawPieChart)

    return () => {
      window.removeEventListener("resize", setCanvasSize)
      window.removeEventListener("resize", drawPieChart)
    }
  }, [])

  return (
    <div className="w-full max-w-2xl mx-auto">
      <canvas ref={canvasRef} className="w-full"></canvas>
    </div>
  )
}
