"use client";

import React, { ReactNode } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Check, X, Info } from "lucide-react";
import { cn } from "@/lib/utils";

// 定义对话框类型
export type DialogType =
  | "confirm"
  | "success"
  | "error"
  | "stakeInput"
  | "welcome"
  | "network";

// 定义组件接口
interface DialogModalProps {
  open: boolean;
  title: string;
  description?: string | ReactNode;
  type: DialogType;
  confirmAction?: (amount?: number) => void;
  confirmText?: string;
  cancelText?: string;
  data?: any;
  isLoading?: boolean;
  onOpenChange: (open: boolean) => void;
  hideCancel?: boolean;
  preventClose?: boolean;
  hideCloseButton?: boolean;
}

export default function DialogModal({
  open,
  title,
  description,
  type,
  confirmAction,
  confirmText = "Confirm",
  cancelText = "Cancel",
  data,
  isLoading = false,
  onOpenChange,
  hideCancel = false,
  preventClose = false,
  hideCloseButton = false,
}: DialogModalProps) {
  // 处理对话框关闭
  const handleOpenChange = (newOpen: boolean) => {
    if (preventClose && !newOpen) {
      return;
    }
    onOpenChange(newOpen);
  };

  // 渲染对话框内容
  const renderDialogContent = () => {
    switch (type) {
      case "success":
        return (
          <div className="flex flex-col items-center justify-center py-4">
            <div className="rounded-full bg-green-600/20 p-3 mb-4">
              <Check className="h-8 w-8 text-green-600" />
            </div>
            <DialogDescription className="text-center">
              {description}
            </DialogDescription>
          </div>
        );
      case "error":
        return (
          <div className="flex flex-col items-center justify-center py-4">
            <div className="rounded-full bg-red-600/20 p-3 mb-4">
              <X className="h-8 w-8 text-red-600" />
            </div>
            <DialogDescription className="text-center">
              {description}
            </DialogDescription>
          </div>
        );
      case "welcome":
      case "network":
        return (
          <div className="flex flex-col items-center justify-center py-4">
            <div className="rounded-full bg-purple-600/20 p-3 mb-4">
              <Info className="h-8 w-8 text-purple-600" />
            </div>
            <div className="text-center">{description}</div>
          </div>
        );
      case "stakeInput":
        return (
          <div className="py-4">
            <DialogDescription className="text-center mb-4">
              {description}
            </DialogDescription>
            <div className="flex items-center justify-center gap-2">
              <input
                type="number"
                min="1"
                max={data?.maxAmount || 1}
                defaultValue="1"
                className="w-20 px-2 py-1 text-center rounded-md border border-purple-500/30 bg-black/30 text-white"
                id="stakeAmount"
              />
              <span className="text-sm text-purple-200">cards</span>
            </div>
          </div>
        );
      case "confirm":
      default:
        return (
          <div className="py-4">
            <DialogDescription className="text-center">
              {description}
            </DialogDescription>
          </div>
        );
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent
        className={cn(
          "bg-gradient-to-br from-purple-900 to-indigo-900 border-purple-500/30 text-white",
          hideCloseButton && "!pr-6" // 如果隐藏关闭按钮，则移除右侧padding
        )}
        hideCloseButton={hideCloseButton}
      >
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>

        {renderDialogContent()}

        <DialogFooter className="flex sm:justify-center gap-2">
          {type === "network" ? (
            <Button
              onClick={() => confirmAction?.()}
              className="bg-gradient-to-r from-purple-600 to-pink-600"
              disabled={isLoading}
            >
              {isLoading ? "Processing..." : confirmText}
            </Button>
          ) : type === "welcome" ? (
            <Button
              onClick={() => confirmAction?.()}
              className="bg-gradient-to-r from-purple-600 to-pink-600"
              disabled={isLoading}
            >
              {isLoading ? "Processing..." : confirmText}
            </Button>
          ) : (
            <>
              {!hideCancel && (
                <Button
                  variant="outline"
                  onClick={() => onOpenChange(false)}
                  className="border-purple-500 text-purple-200 bg-purple-950/50 hover:bg-purple-800/50 hover:text-white"
                  disabled={isLoading}
                >
                  {cancelText}
                </Button>
              )}
              <Button
                onClick={() => {
                  if (type === "stakeInput" && confirmAction) {
                    const inputEl = document.getElementById(
                      "stakeAmount"
                    ) as HTMLInputElement;
                    const amount = Number.parseInt(inputEl.value);
                    confirmAction(amount);
                  } else if (confirmAction) {
                    confirmAction();
                  }
                }}
                className="bg-gradient-to-r from-purple-600 to-pink-600"
                disabled={isLoading}
              >
                {isLoading ? "Processing..." : confirmText}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
