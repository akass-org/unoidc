import type { ReactNode, CSSProperties } from 'react'

interface CardProps {
  children: ReactNode
  className?: string
  padding?: 'none' | 'sm' | 'md' | 'lg'
  hover?: boolean
  style?: CSSProperties
}

export function Card({ children, className = '', padding = 'md', hover = false, style }: CardProps) {
  const paddings = {
    none: '',
    sm: 'p-4',
    md: 'p-5',
    lg: 'p-6',
  }

  return (
    <div
      className={`
        bg-white dark:bg-white/[0.02] backdrop-blur-sm border border-gray-200 dark:border-white/[0.06] rounded-xl shadow-sm dark:shadow-none
        ${paddings[padding]}
        ${hover ? 'card-hover hover:border-gray-300 dark:hover:border-white/[0.12] hover:bg-gray-50 dark:hover:bg-white/[0.04]' : 'transition-all duration-200'}
        ${className}
      `}
      style={style}
    >
      {children}
    </div>
  )
}

interface CardHeaderProps {
  title: string | ReactNode
  subtitle?: string
  action?: ReactNode
}

export function CardHeader({ title, subtitle, action }: CardHeaderProps) {
  return (
    <div className="flex items-start justify-between mb-5">
      <div>
        {typeof title === 'string' ? (
          <h3 className="text-base font-bold text-gray-900 dark:text-white">{title}</h3>
        ) : (
          <h3 className="text-base text-gray-900 dark:text-white">{title}</h3>
        )}
        {subtitle && <p className="text-sm text-gray-500 mt-0.5">{subtitle}</p>}
      </div>
      {action}
    </div>
  )
}
