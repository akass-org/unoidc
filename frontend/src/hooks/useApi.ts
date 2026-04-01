import { useState, useCallback } from 'react'
import { useToast } from '#src/components/ui'
import { getErrorMessage, type ApiError } from '#src/api/client'

interface UseApiOptions<T> {
  onSuccess?: (data: T) => void
  onError?: (error: ApiError) => void
  successMessage?: string
  errorMessage?: string
}

export function useApi<T, Args extends unknown[]>(
  apiFn: (...args: Args) => Promise<T>,
  options: UseApiOptions<T> = {}
) {
  const { onSuccess, onError, successMessage, errorMessage } = options
  const { addToast } = useToast()
  
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<ApiError | null>(null)

  const execute = useCallback(
    async (...args: Args) => {
      setLoading(true)
      setError(null)
      
      try {
        const result = await apiFn(...args)
        setData(result)
        
        if (successMessage) {
          addToast({
            type: 'success',
            title: successMessage,
          })
        }
        
        onSuccess?.(result)
        return result
      } catch (err) {
        const apiError = err as ApiError
        setError(apiError)
        
        const message = errorMessage || getErrorMessage(err)
        addToast({
          type: 'error',
          title: '操作失败',
          message,
        })
        
        onError?.(apiError)
        throw err
      } finally {
        setLoading(false)
      }
    },
    [apiFn, onSuccess, onError, successMessage, errorMessage, addToast]
  )

  const reset = useCallback(() => {
    setData(null)
    setError(null)
    setLoading(false)
  }, [])

  return { data, loading, error, execute, reset }
}

// Hook for data fetching (GET requests)
export function useFetch<T>(fetchFn: () => Promise<T>, _deps: unknown[] = []) {
  const { addToast } = useToast()
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<ApiError | null>(null)

  const fetch = useCallback(async () => {
    setLoading(true)
    setError(null)
    
    try {
      const result = await fetchFn()
      setData(result)
      return result
    } catch (err) {
      setError(err as ApiError)
      addToast({
        type: 'error',
        title: '加载失败',
        message: getErrorMessage(err),
      })
      throw err
    } finally {
      setLoading(false)
    }
  }, [fetchFn, addToast])

  // Auto-fetch on mount and deps change
  useState(() => {
    fetch()
  })

  return { data, loading, error, refetch: fetch, setData }
}
