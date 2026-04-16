import { useEffect, useState, type ReactNode } from "react";
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from "lucide-react";

type ToastType = "success" | "error" | "info" | "warning";

interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
}

interface ToastContextType {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, "id">) => void;
  removeToast: (id: string) => void;
}

import { createContext, useContext, useCallback } from "react";

const ToastContext = createContext<ToastContextType | undefined>(undefined);

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback((toast: Omit<Toast, "id">) => {
    const id = Math.random().toString(36).substring(2, 9);
    setToasts((prev) => [...prev, { ...toast, id }]);
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ toasts, addToast, removeToast }}>
      {children}
      <ToastContainer toasts={toasts} onRemove={removeToast} />
    </ToastContext.Provider>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within ToastProvider");
  }
  return context;
}

const icons = {
  success: CheckCircle,
  error: AlertCircle,
  info: Info,
  warning: AlertTriangle,
};

const styles = {
  success:
    "bg-gray-100 dark:bg-white/[0.04] border-gray-200 dark:border-white/[0.08] text-gray-900 dark:text-white",
  error:
    "bg-red-50 dark:bg-red-500/[0.08] border-red-200 dark:border-red-500/[0.16] text-red-600 dark:text-red-400",
  info: "bg-blue-50 dark:bg-blue-500/[0.08] border-blue-200 dark:border-blue-500/[0.16] text-blue-600 dark:text-blue-400",
  warning:
    "bg-amber-50 dark:bg-amber-500/[0.08] border-amber-200 dark:border-amber-500/[0.16] text-amber-600 dark:text-amber-400",
};

function ToastItem({ toast, onRemove }: { toast: Toast; onRemove: (id: string) => void }) {
  const [isExiting, setIsExiting] = useState(false);
  const Icon = icons[toast.type];

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsExiting(true);
      setTimeout(() => onRemove(toast.id), 200);
    }, 4000);
    return () => clearTimeout(timer);
  }, [toast.id, onRemove]);

  const handleClose = () => {
    setIsExiting(true);
    setTimeout(() => onRemove(toast.id), 200);
  };

  return (
    <div
      className={`
        flex items-start gap-3 p-4 rounded-lg border backdrop-blur-md shadow-xl
        ${styles[toast.type]}
        ${isExiting ? "animate-out fade-out slide-out-to-right" : "animate-in fade-in slide-in-from-right"}
        transition-all duration-200
      `}
    >
      <Icon className="w-5 h-5 flex-shrink-0 mt-0.5" />
      <div className="flex-1 min-w-0">
        <p className="font-bold text-sm">{toast.title}</p>
        {toast.message && <p className="text-sm opacity-70 mt-0.5">{toast.message}</p>}
      </div>
      <button
        onClick={handleClose}
        className="flex-shrink-0 opacity-50 hover:opacity-100 transition-opacity"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}

function ToastContainer({ toasts, onRemove }: { toasts: Toast[]; onRemove: (id: string) => void }) {
  return (
    <div className="fixed top-4 right-4 z-[100] flex flex-col gap-2 w-full max-w-sm pointer-events-none">
      {toasts.map((toast) => (
        <div key={toast.id} className="pointer-events-auto">
          <ToastItem toast={toast} onRemove={onRemove} />
        </div>
      ))}
    </div>
  );
}
