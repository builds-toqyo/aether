"use client";

import React, { createContext, useContext, useState, useCallback } from "react";
import { X, Info, AlertTriangle, CheckCircle } from "lucide-react";

type ToastType = "info" | "success" | "warning" | "error";

interface Toast {
  id: string;
  message: string;
  type: ToastType;
}

interface ToastContextValue {
  toast: (message: string, type?: ToastType) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export const useToast = (): ToastContextValue => {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be inside ToastProvider");
  return ctx;
};

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const toast = useCallback((message: string, type: ToastType = "info") => {
    const id = `${Date.now()}_${Math.random()}`;
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 3000);
  }, []);

  const remove = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const iconMap: Record<ToastType, React.ReactNode> = {
    info: <Info size={14} className="text-blue-400" />,
    success: <CheckCircle size={14} className="text-green-400" />,
    warning: <AlertTriangle size={14} className="text-yellow-400" />,
    error: <X size={14} className="text-red-400" />,
  };

  const bgMap: Record<ToastType, string> = {
    info: "bg-blue-900/30 border-blue-800/50",
    success: "bg-green-900/30 border-green-800/50",
    warning: "bg-yellow-900/30 border-yellow-800/50",
    error: "bg-red-900/30 border-red-800/50",
  };

  return (
    <ToastContext.Provider value={{ toast }}>
      {children}
      <div className="fixed bottom-8 left-1/2 -translate-x-1/2 flex flex-col gap-2 z-[100]">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={`flex items-center gap-2 px-4 py-2 rounded-md border ${bgMap[t.type]} shadow-lg backdrop-blur-sm min-w-[200px]`}
          >
            {iconMap[t.type]}
            <span className="text-[12px] text-[#e0e0e0]">{t.message}</span>
            <button onClick={() => remove(t.id)} className="ml-auto text-[#555] hover:text-white">
              <X size={12} />
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
};
