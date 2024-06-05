import { createLazyFileRoute } from '@tanstack/react-router'
import App from '../../App'

export const Route = createLazyFileRoute('/app/test')({
  component: Test,
})

function Test() {
  return <App />
}