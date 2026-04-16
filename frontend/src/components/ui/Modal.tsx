import { useEffect, useRef, type ReactNode } from "react";
import { X } from "lucide-react";
import { Button } from "./Button";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  description?: string;
  children: ReactNode;
  footer?: ReactNode;
  size?: "sm" | "md" | "lg" | "xl";
}

export function Modal({
  isOpen,
  onClose,
  title,
  description,
  children,
  footer,
  size = "md",
}: ModalProps) {
  const overlayRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
      document.body.style.overflow = "hidden";
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
      document.body.style.overflow = "";
    };
  }, [isOpen, onClose]);

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === overlayRef.current) {
      onClose();
    }
  };

  const sizes = {
    sm: "max-w-md",
    md: "max-w-lg",
    lg: "max-w-2xl",
    xl: "max-w-4xl",
  };

  if (!isOpen) return null;

  return (
    <div
      ref={overlayRef}
      onClick={handleOverlayClick}
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 dark:bg-black/70 backdrop-blur-sm animate-in fade-in duration-200"
    >
      <div
        className={`
          w-full ${sizes[size]} 
          bg-white dark:bg-[#0a0a0a] border border-gray-200 dark:border-white/[0.08] rounded-xl shadow-2xl
          animate-in zoom-in-95 duration-200
        `}
      >
        <div className="flex items-start justify-between p-5 border-b border-gray-200 dark:border-white/[0.06]">
          <div>
            <h2 className="text-base font-bold text-gray-900 dark:text-white">{title}</h2>
            {description && <p className="text-sm text-gray-500 mt-1">{description}</p>}
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/[0.05] rounded-md transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-5">{children}</div>

        {footer && (
          <div className="flex items-center justify-end gap-2 px-5 py-4 border-t border-gray-200 dark:border-white/[0.06] bg-gray-50 dark:bg-white/[0.02] rounded-b-xl">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
}

interface ConfirmModalProps extends Omit<ModalProps, "children" | "footer"> {
  onConfirm: () => void;
  confirmText?: string;
  cancelText?: string;
  variant?: "danger" | "primary";
  loading?: boolean;
}

export function ConfirmModal({
  onConfirm,
  confirmText = "确认",
  cancelText = "取消",
  variant = "primary",
  loading,
  ...props
}: ConfirmModalProps) {
  return (
    <Modal
      {...props}
      footer={
        <>
          <Button variant="ghost" onClick={props.onClose}>
            {cancelText}
          </Button>
          <Button
            variant={variant === "danger" ? "danger" : "primary"}
            onClick={onConfirm}
            loading={loading}
          >
            {confirmText}
          </Button>
        </>
      }
    >
      <p className="text-gray-600 dark:text-gray-400 text-sm">{props.description}</p>
    </Modal>
  );
}
