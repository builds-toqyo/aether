import "./globals.css";
import type { ReactNode } from "react";
import { ToastProvider } from "@/components/Toast";

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <ToastProvider>{children}</ToastProvider>
      </body>
    </html>
  );
}
