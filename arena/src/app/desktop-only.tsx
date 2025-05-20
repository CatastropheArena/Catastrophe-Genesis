import {useEffect, useState} from "react";

import {Card} from "@entities/card";
import {Layout} from "@shared/lib/layout";
import {H3, Text} from "@shared/ui/atoms";
import {useTheme} from "@shared/lib/theming";

export const DesktopOnlyRestrict: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  const [showWarning, setShowWarning] = useState(false);
  const [dismissed, setDismissed] = useState(false);
  const { mode } = useTheme();

  useEffect(() => {
    const adjust = () => {
      const width = window.innerWidth; // Using innerWidth instead of document.body.clientWidth
      const minWidth = 993; // Increased minimum width to a more standard mobile breakpoint
      // Show warning when the width is less than minimum width and not dismissed
      setShowWarning(width < minWidth && !dismissed);
    };

    adjust();

    window.addEventListener("resize", adjust);

    return () => {
      window.removeEventListener("resize", adjust);
    };
  }, [dismissed]);

  const handleDismiss = () => {
    setDismissed(true);
    setShowWarning(false);
  };

  return (
    <>
      {children}
      
      {showWarning && (
        <>
          {/* Backdrop */}
          <div 
            className="fixed inset-0 bg-black/50 z-[999] animate-fade-in"
            onClick={handleDismiss}
          />
          
          {/* Warning Overlay */}
          <div className="
            fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2
            bg-gray-50/95 dark:bg-black/85
            rounded-lg p-3 z-[1000] max-w-[420px] w-[90%]
            shadow-[0_8px_24px_rgba(0,0,0,0.2)] dark:shadow-[0_8px_24px_rgba(0,0,0,0.6)]
            border border-gray-200 dark:border-gray-500
            ring-1 ring-gray-300 dark:ring-gray-500
            animate-[fadeIn_0.4s_ease-out]
            text-black dark:text-white
          ">
            {/* Close Button */}
            <button 
              className="
                absolute top-3 right-3 bg-transparent border-none 
                text-gray-500 dark:text-gray-400
                text-5xl cursor-pointer 
                hover:text-black dark:hover:text-white
              "
              onClick={handleDismiss}
            >
              ×
            </button>
            
            <Layout.Row align="center" gap={3}>
              <Card name="nope" />

              <Layout.Col w={30} gap={2}>
                <H3>Screen Width Warning</H3>

                <Text emphasis="secondary">
                  Your screen width is too narrow, which may affect your experience. 
                  We recommend using a device with a larger screen.
                </Text>
              </Layout.Col>
            </Layout.Row>
          </div>
        </>
      )}
    </>
  );
};
