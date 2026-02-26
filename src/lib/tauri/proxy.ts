import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Proxy management
export interface ProxyStatus {
  endpoint: string;
  port: number;
  running: boolean;
}

export interface RotationStatus {
  active: boolean;
  providerId?: string;
  providerName?: string;
  currentProxy: string;
  /** Real external IP address (dynamic, changes on rotation) */
  realIp: string;
  expiresInSeconds: number;
  ttlSeconds: number;
}

// Provider types for multi-provider proxy rotation
export interface ProviderInfo {
  id: string;
  name: string;
}

export interface NetworkOption {
  value: string;
  label: string;
}

export interface LocationOption {
  value: string;
  label: string;
}

export interface ProviderMetadata {
  id: string;
  name: string;
  requiredFields: string[];
  networkOptions: NetworkOption[];
  locationOptions: LocationOption[];
}

export interface RotationProxySettings {
  providerId: string;
  apiKey: string;
  networkType: string;
  locationFilter: string;
  customApiUrl?: string;
}

export async function startProxy(): Promise<ProxyStatus> {
  return invoke("start_proxy");
}

export async function stopProxy(): Promise<ProxyStatus> {
  return invoke("stop_proxy");
}

export async function getProxyStatus(): Promise<ProxyStatus> {
  return invoke("get_proxy_status");
}

export async function onProxyStatusChanged(
  callback: (status: ProxyStatus) => void,
): Promise<UnlistenFn> {
  return listen<ProxyStatus>("proxy-status-changed", (event) => {
    callback(event.payload);
  });
}

export async function onTrayToggleProxy(
  callback: (shouldStart: boolean) => void,
): Promise<UnlistenFn> {
  return listen<boolean>("tray-toggle-proxy", (event) => {
    callback(event.payload);
  });
}

// Rotation proxy management
export async function getRotationStatus(): Promise<RotationStatus> {
  return invoke("get_rotation_status");
}

export async function forceRotateProxy(): Promise<RotationStatus> {
  return invoke("force_rotate_proxy");
}

export async function onRotationProxyUpdated(
  callback: (data: {
    proxy: string;
    realIp?: string;
    ttl: number;
    expiresInSeconds?: number;
  }) => void,
): Promise<UnlistenFn> {
  return listen<{ proxy: string; realIp?: string; ttl: number; expiresInSeconds?: number }>(
    "rotation-proxy-updated",
    (event) => {
      callback(event.payload);
    },
  );
}

export async function onRotationProxyError(
  callback: (data: { error: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ error: string }>("rotation-proxy-error", (event) => {
    callback(event.payload);
  });
}

export async function onRotationProxyActive(
  callback: (data: { active: boolean }) => void,
): Promise<UnlistenFn> {
  return listen<{ active: boolean }>("rotation-proxy-active", (event) => {
    callback(event.payload);
  });
}

// Multi-provider proxy rotation API
export async function getAvailableRotationProviders(): Promise<ProviderInfo[]> {
  return invoke("get_available_rotation_providers");
}

export async function getProviderMetadata(providerId: string): Promise<ProviderMetadata> {
  return invoke("get_provider_metadata", { providerId });
}

export async function updateRotationSettings(settings: RotationProxySettings): Promise<void> {
  return invoke("update_rotation_settings", { settings });
}
