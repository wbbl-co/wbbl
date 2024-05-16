/* prettier-ignore-start */

/* eslint-disable */

// @ts-nocheck

// noinspection JSUnusedGlobalSymbols

// This file is auto-generated by TanStack Router

import { createFileRoute } from '@tanstack/react-router'

// Import Routes

import { Route as rootRoute } from './routes/__root'

// Create Virtual Routes

const AppIndexLazyImport = createFileRoute('/app/')()
const AppAboutLazyImport = createFileRoute('/app/about')()

// Create/Update Routes

const AppIndexLazyRoute = AppIndexLazyImport.update({
  path: '/app/',
  getParentRoute: () => rootRoute,
} as any).lazy(() => import('./routes/app/index.lazy').then((d) => d.Route))

const AppAboutLazyRoute = AppAboutLazyImport.update({
  path: '/app/about',
  getParentRoute: () => rootRoute,
} as any).lazy(() => import('./routes/app/about.lazy').then((d) => d.Route))

// Populate the FileRoutesByPath interface

declare module '@tanstack/react-router' {
  interface FileRoutesByPath {
    '/app/about': {
      id: '/app/about'
      path: '/app/about'
      fullPath: '/app/about'
      preLoaderRoute: typeof AppAboutLazyImport
      parentRoute: typeof rootRoute
    }
    '/app/': {
      id: '/app/'
      path: '/app'
      fullPath: '/app'
      preLoaderRoute: typeof AppIndexLazyImport
      parentRoute: typeof rootRoute
    }
  }
}

// Create and export the route tree

export const routeTree = rootRoute.addChildren({
  AppAboutLazyRoute,
  AppIndexLazyRoute,
})

/* prettier-ignore-end */
