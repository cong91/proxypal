import { createRoot, createSignal } from "solid-js";
import type { RotationStatus, RotationProxySettings } from "../lib/tauri/proxy";

interface CachedRotationState {
  status: RotationStatus | null;
  lastFetchedAt: number | null;
  settings: RotationProxySettings | null;
  providerId: string | null;
}

function createRotationStore() {
  // Cached rotation status for instant display when navigating back to Settings
  const [cachedState, setCachedState] = createSignal<CachedRotationState>({
    status: null,
    lastFetchedAt: null,
    settings: null,
    providerId: null,
  });

  // Cache the rotation status after successful fetch
  const cacheStatus = (status: RotationStatus) => {
    setCachedState((prev) => ({
      ...prev,
      status,
      lastFetchedAt: Date.now(),
    }));
  };

  // Cache the rotation settings
  const cacheSettings = (settings: RotationProxySettings | null, providerId?: string) => {
    setCachedState((prev) => ({
      ...prev,
      settings,
      providerId: providerId || prev.providerId,
    }));
  };

  // Clear the cache (e.g., when proxy stops or on error)
  const clearCache = () => {
    setCachedState({
      status: null,
      lastFetchedAt: null,
      settings: null,
      providerId: null,
    });
  };

  // Get cached status if it's still fresh (within 30 seconds)
  const getFreshStatus = (): RotationStatus | null => {
    const state = cachedState();
    if (!state.status || !state.lastFetchedAt) return null;
    
    // Consider cache fresh if less than 30 seconds old
    const isFresh = Date.now() - state.lastFetchedAt < 30000;
    return isFresh ? state.status : null;
  };

  // Check if we have any cached data
  const hasCachedData = (): boolean => {
    return cachedState().status !== null;
  };

  return {
    cachedState,
    cacheStatus,
    cacheSettings,
    clearCache,
    getFreshStatus,
    hasCachedData,
  };
}

export const rotationStore = createRoot(createRotationStore);
