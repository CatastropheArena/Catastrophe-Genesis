/** @type {import('next').NextConfig} */
const defaultConfig = {
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  images: {
    unoptimized: true,
  },
  experimental: {
    webpackBuildWorker: true,
    parallelServerBuildTraces: true,
    parallelServerCompiles: true,
  },
}

/**
 * 深度合并两个配置对象
 * @param {object} target - 目标配置对象
 * @param {object} source - 源配置对象
 * @returns {object} - 合并后的配置对象
 */
function deepMerge(target, source) {
  if (!source) {
    return target
  }

  const merged = { ...target }
  
  Object.keys(source).forEach((key) => {
    if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
      merged[key] = deepMerge(merged[key] || {}, source[key])
    } else {
      merged[key] = source[key]
    }
  })

  return merged
}

// 如果存在自定义配置，可以在这里导入并合并
let finalConfig = defaultConfig

try {
  const { default: userConfig } = await import('./next.config.local.js')
  if (userConfig) {
    finalConfig = deepMerge(defaultConfig, userConfig)
  }
} catch (e) {
  // 如果本地配置文件不存在，使用默认配置
  console.log('Using default Next.js configuration')
}

export default finalConfig
