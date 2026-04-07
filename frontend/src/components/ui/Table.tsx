import type { ReactNode } from 'react'

interface TableProps<T> {
  data: T[]
  columns: Column<T>[]
  keyExtractor: (item: T) => string
  emptyState?: ReactNode
  loading?: boolean
  tableMinWidth?: string
}

interface Column<T> {
  key: string
  title: string
  render: (item: T) => ReactNode
  width?: string
}

export function Table<T>({
  data,
  columns,
  keyExtractor,
  emptyState,
  loading,
  tableMinWidth,
}: TableProps<T>) {
  if (loading) {
    return (
      <div className="w-full h-48 flex items-center justify-center">
        <div className="w-6 h-6 border border-gray-300 dark:border-white/20 border-t-gray-900 dark:border-t-white rounded-full animate-spin" />
      </div>
    )
  }

  if (data.length === 0 && emptyState) {
    return <div className="py-12">{emptyState}</div>
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full" style={tableMinWidth ? { minWidth: tableMinWidth } : undefined}>
        <thead>
          <tr className="border-b border-gray-200 dark:border-white/[0.06]">
            {columns.map((col) => (
              <th
                key={col.key}
                className="text-left py-3 px-4 text-xs font-medium text-gray-500 uppercase tracking-wider"
                style={{ width: col.width }}
              >
                {col.title}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200 dark:divide-white/[0.04]">
          {data.map((item) => (
            <tr
              key={keyExtractor(item)}
              className="hover:bg-gray-50 dark:hover:bg-white/[0.02] transition-colors"
            >
              {columns.map((col) => (
                <td key={col.key} className="py-3.5 px-4">
                  {col.render(item)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

export function EmptyState({
  icon,
  title,
  description,
  action,
}: {
  icon: ReactNode
  title: string
  description: string
  action?: ReactNode
}) {
  return (
    <div className="text-center py-12">
      <div className="w-12 h-12 mx-auto mb-4 rounded-lg bg-gray-100 dark:bg-white/[0.04] flex items-center justify-center text-gray-500 dark:text-gray-600 border border-gray-200 dark:border-white/[0.06]">
        {icon}
      </div>
      <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">{title}</h3>
      <p className="text-sm text-gray-500 dark:text-gray-600 mb-4">{description}</p>
      {action}
    </div>
  )
}
