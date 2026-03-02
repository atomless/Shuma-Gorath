import { base } from '$app/paths';
import {
  normalizeDashboardBasePath,
  resolveDashboardAssetPath
} from '$lib/runtime/dashboard-paths.js';

export function load({ url }) {
  const dashboardBasePath = normalizeDashboardBasePath(base);
  const initialHashTab = String(url.hash || '').replace(/^#/, '').toLowerCase();

  return {
    dashboardBasePath,
    shumaImageSrc: resolveDashboardAssetPath(
      dashboardBasePath,
      'assets/shuma-gorath-pencil.png'
    ),
    initialHashTab
  };
}
