import ky from 'ky'

// 统一的 API 错误类型
export class ApiError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public responseData?: unknown
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

// 获取用户友好的错误信息
export function getErrorMessage(err: unknown): string {
  // 如果是我们的 ApiError，根据状态码返回不同消息
  if (err instanceof ApiError) {
    // 5xx 服务器错误 - 不暴露技术细节
    if (err.statusCode && err.statusCode >= 500) {
      return '服务器繁忙，请稍后再试'
    }
    // 4xx 客户端错误 - 使用后端返回的错误信息
    if (err.statusCode && err.statusCode >= 400) {
      // 如果有后端返回的具体错误信息，使用它
      if (err.responseData && typeof err.responseData === 'object') {
        const data = err.responseData as Record<string, string>
        if (data.error_description) return data.error_description
        if (data.error) return data.error
        if (data.message) return data.message
      }
      return err.message || '请求失败，请检查输入信息'
    }
    return err.message
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

export const api = ky.create({
  prefixUrl: '/',
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json',
  },
  hooks: {
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
