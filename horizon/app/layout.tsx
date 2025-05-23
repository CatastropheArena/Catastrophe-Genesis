import type { Metadata } from "next";
import "./globals.css";
import { Inter } from "next/font/google";
import { Providers } from "@/app/providers";
import ClientLayout from "@/app/components/layout/ClientLayout";
import { ThemeProvider } from "@/components/theme-provider";

const inter = Inter({ subsets: ["latin"] });

export const metadata = {
  title: "Catastrophe-Genesis - Strategic Card Battle GameFi",
  description:
    "A strategic card-based GameFi platform on the Sui Network where players compete, synthesize cards, and earn rewards",
}
export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
    <body>
      <ThemeProvider attribute="class" defaultTheme="purple" enableSystem={false} themes={["purple", "green"]}>
        <Providers>
          <ClientLayout>{children}</ClientLayout>
        </Providers>
      </ThemeProvider>
    </body>
  </html>


  );
}
