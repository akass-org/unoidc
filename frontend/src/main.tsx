import { StrictMode, useEffect } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import 'virtual:uno.css'
import '@unocss/reset/tailwind.css'
import App from './App.tsx'
import { initTheme } from './stores/theme.ts'

function ThemeInitializer() {
  useEffect(() => {
    initTheme()
  }, [])
  return null
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <ThemeInitializer />
      <App />
    </BrowserRouter>
  </StrictMode>,
)
