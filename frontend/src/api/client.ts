import ky from 'ky'

export const api = ky.create({
  prefixUrl: '/',
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json',
  },
  hooks: {
    beforeError: [
      (error) => {
        // TODO: 统一错误处理
        return error
      },
    ],
  },
})
