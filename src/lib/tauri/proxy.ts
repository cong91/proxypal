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
  currentProxy: string;
  expiresInSeconds: number;
  ttlSeconds: number;
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
  callback: (data: { proxy: string; ttl: number }) => void,
): Promise<UnlistenFn> {
  return listen<{ proxy: string; ttl: number }>(
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
