"use client";

import { useLoading } from "@/context/LoadingContext";
import { Loader2 } from "lucide-react";

export function Loading() {
  const { isLoading, message } = useLoading();

  if (!isLoading) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="flex flex-col items-center gap-3 rounded-lg bg-purple-900/80 p-6 shadow-lg">
        <Loader2 className="h-8 w-8 animate-spin text-purple-300" />
        <p className="text-sm text-purple-100">{message}</p>
      </div>
    </div>
  );
}
