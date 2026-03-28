import { Outlet, NavLink } from 'react-router-dom'

const navItems = [
  { to: '/admin/users', label: '用户管理' },
  { to: '/admin/groups', label: '用户组' },
  { to: '/admin/clients', label: 'Client 管理' },
  { to: '/admin/audit-logs', label: '审计日志' },
  { to: '/admin/settings', label: '系统设置' },
]

export function AdminLayout() {
  return (
    <div className="min-h-screen flex bg-gray-100">
      <aside className="w-56 bg-white shadow p-4">
        <h2 className="text-lg font-bold mb-4">管理后台</h2>
        <nav className="flex flex-col gap-2">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                `px-3 py-2 rounded text-sm ${isActive ? 'bg-blue-500 text-white' : 'hover:bg-gray-100'}`
              }
            >
              {item.label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="flex-1 p-6">
        <Outlet />
      </main>
    </div>
  )
}
