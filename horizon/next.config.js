/** @type {import('next').NextConfig} */
const nextConfig = {
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
  transpilePackages: ["@cetusprotocol/terminal"],
  webpack: (config) => {
    // 让 .es.js 也走 babel-loader
    config.module.rules.push({
      test: /\.es\.js$/,
      include: /node_modules\/@cetusprotocol\/terminal/,
      use: {
        loader: "babel-loader",
        options: {
          presets: ["next/babel"],
        },
      },
    });
    return config;
  },
};

module.exports = nextConfig;
