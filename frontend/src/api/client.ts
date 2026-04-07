import ky from 'ky'

// 统一的 API 错误类型
export class ApiError extends Error {
  statusCode?: number
  responseData?: unknown
  
  constructor(
    message: string,
    statusCode?: number,
    responseData?: unknown
  ) {
    super(message)
    this.name = 'ApiError'
    this.statusCode = statusCode
    this.responseData = responseData
  }
}

// 获取用户友好的错误信息
// 注意：任何情况下都不把原始错误 message 暴露给用户
export function getErrorMessage(err: unknown, context?: 'login' | 'default'): string {
  const ctx = context || 'default'
  
  // 如果是我们的 ApiError，根据状态码返回通用友好消息
  if (err instanceof ApiError) {
    // 5xx 服务器错误
    if (err.statusCode && err.statusCode >= 500) {
      return '服务器繁忙，请稍后再试'
    }
    // 401 未授权
    if (err.statusCode === 401) {
      // 登录场景下，401 表示用户名/密码错误
      if (ctx === 'login') {
        return '用户名或密码错误'
      }
      return '登录已过期，请重新登录'
    }
    // 403 禁止访问
    if (err.statusCode === 403) {
      return '您没有权限执行此操作'
    }
    // 404 未找到
    if (err.statusCode === 404) {
      return '请求的资源不存在'
    }
    // 409 冲突（如用户名已存在）
    if (err.statusCode === 409) {
      return '该用户名或邮箱已被使用'
    }
    // 422 验证错误
    if (err.statusCode === 422) {
      return '输入信息有误，请检查后重试'
    }
    // 其他 4xx 客户端错误
    if (err.statusCode && err.statusCode >= 400) {
      return '请求失败，请检查输入信息后重试'
    }
    return '操作失败，请稍后重试'
  }

  // 原生 Error
  if (err instanceof Error) {
    // 网络错误
    if (err.message === 'Failed to fetch' || err.message.includes('NetworkError')) {
      return '无法连接到服务器，请检查网络或稍后再试'
    }
    // 其他错误，返回通用消息（不暴露原始错误详情）
    return '操作失败，请稍后重试'
  }

  return '操作失败，请稍后重试'
}

// 从 cookie 中获取指定名称的值
function getCookie(name: string): string | undefined {
  const value = `; ${document.cookie}`
  const parts = value.split(`; ${name}=`)
  if (parts.length === 2) return parts.pop()?.split(';').shift()
}

export const api = ky.create({
  prefixUrl: '/',
  credentials: 'include',
  hooks: {
    beforeRequest: [
      (request) => {
        // 对于修改状态的请求，必须携带 CSRF token
        const method = request.method.toUpperCase()
        if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(method)) {
          const csrfToken = getCookie('unoidc_csrf')
          if (csrfToken) {
            request.headers.set('x-csrf-token', csrfToken)
          } else {
            // CSRF token 缺失时阻止修改请求，防止 CSRF 保护被绕过
            console.warn('[CSRF] CSRF token missing for mutating request - server will reject')
          }
        }
      },
    ],
    beforeError: [
      async (error) => {
        const { response } = error
        let responseData: unknown = undefined

        // 尝试解析后端返回的错误信息
        try {
          responseData = await response.clone().json()
        } catch {
          try {
            responseData = await response.clone().text()
          } catch {
            // 忽略解析错误
          }
        }

        throw new ApiError(
          error.message,
          response.status,
          responseData
        )
      },
    ],
  },
})
