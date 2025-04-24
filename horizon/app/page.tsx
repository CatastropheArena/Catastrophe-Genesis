"use client";

import { useCurrentAccount } from "@mysten/dapp-kit";
import { usePathname } from "next/navigation";
import dynamic from "next/dynamic";

// 使用动态导入来避免 SSR 问题
const HomePage = dynamic(() => import("@/app/components/home/HomePage"), {
  ssr: false,
});

const Dashboard = dynamic(
  () => import("@/app/components/dashboard/Dashboard"),
  {
    ssr: false,
  }
);

export default function Page() {
  const account = useCurrentAccount();
  const pathname = usePathname();

  if (pathname === "/" && account && account?.address) {
    return <Dashboard />;
  }

  return <HomePage />;
}
