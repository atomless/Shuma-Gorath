import { base } from '$app/paths';
import {
  normalizeDashboardBasePath,
  resolveDashboardAssetPath
} from '$lib/runtime/dashboard-paths.js';

export function load() {
  const dashboardBasePath = normalizeDashboardBasePath(base);

  return {
    dashboardBasePath,
    faviconHref: resolveDashboardAssetPath(
      dashboardBasePath,
      'assets/shuma-gorath-pencil-closed.png'
    )
  };
}
