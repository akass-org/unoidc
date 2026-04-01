import type { InputHTMLAttributes, ReactNode } from 'react'
import { forwardRef } from 'react'

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string
  error?: string
  icon?: ReactNode
  helper?: string
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, icon, helper, className = '', ...props }, ref) => {
    return (
      <div className="w-full">
        {label && (
          <label className="block text-sm font-medium text-gray-600 dark:text-gray-400 mb-1.5">
            {label}
          </label>
        )}
        <div className="relative group">
          {icon && (
            <div className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-gray-300 transition-colors">
              {icon}
            </div>
          )}
          <input
            ref={ref}
            className={`
              w-full bg-gray-50 dark:bg-white/[0.04] border rounded-lg px-4 py-2.5
              text-gray-900 dark:text-white placeholder:text-gray-400 dark:placeholder:text-gray-600
              focus:outline-none focus:ring-1 focus:ring-black/10 dark:focus:ring-white/20 focus:border-black/20 dark:focus:border-white/30 focus:bg-white dark:focus:bg-white/[0.06]
              hover:border-gray-300 dark:hover:border-white/20 hover:bg-white dark:hover:bg-white/[0.05]
              transition-all duration-200
              ${error ? 'border-red-500/40 focus:border-red-500/60 focus:ring-red-500/20' : 'border-gray-200 dark:border-white/[0.08]'}
              ${icon ? 'pl-10' : ''}
              ${className}
            `}
            {...props}
          />
        </div>
        {error && (
          <p className="mt-1.5 text-xs text-red-400">{error}</p>
        )}
        {helper && !error && (
          <p className="mt-1.5 text-xs text-gray-500 dark:text-gray-600">{helper}</p>
        )}
      </div>
    )
  }
)

Input.displayName = 'Input'
