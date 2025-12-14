/**
 * LazyRaceComponents - Lazy-loaded wrappers for race interface components
 *
 * This module provides lazy-loaded versions of race components to improve
 * initial bundle size and loading performance.
 *
 * Requirements: 12.5
 */

import { lazy } from 'react';

// Lazy load performance-heavy components
export const LazyPerformancePreview = lazy(() =>
  import('./PerformancePreview').then((module) => ({ default: module.PerformancePreview })),
);

export const LazyPlayerCarCard = lazy(() =>
  import('./PlayerCarCard').then((module) => ({ default: module.PlayerCarCard })),
);

export const LazyLocalSectorDisplay = lazy(() =>
  import('./LocalSectorDisplay').then((module) => ({ default: module.LocalSectorDisplay })),
);

export const LazyRaceStatusPanel = lazy(() =>
  import('./RaceStatusPanel').then((module) => ({ default: module.RaceStatusPanel })),
);

export const LazyBoostSelector = lazy(() =>
  import('./BoostSelector').then((module) => ({ default: module.BoostSelector })),
);

// Lazy load the complete race interface for code splitting
export const LazyRaceInterface = lazy(() => import('./RaceInterface'));

// Export loading fallback component
export { RaceLoadingState as LazyLoadingFallback } from './RaceLoadingState';
