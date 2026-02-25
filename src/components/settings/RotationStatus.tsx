import { createEffect, createSignal, on, onCleanup, Show } from "solid-js";
import {
  forceRotateProxy,
  getRotationStatus,
  onRotationProxyActive,
  onRotationProxyError,
  onRotationProxyInitialized,
  onRotationProxyUpdated,
  type RotationStatus as RotationStatusType,
} from "../../lib/tauri/proxy";
import { rotationStore } from "../../stores/rotation";
import { toastStore } from "../../stores/toast";

interface RotationStatusProps {
  isActive: boolean;
}

export function RotationStatus(props: RotationStatusProps) {
  // Initialize from cache for instant display
  const cachedStatus = rotationStore.cachedState().status;
  const cachedTimeLeft = cachedStatus?.expiresInSeconds ?? 0;
  
  const [status, setStatus] = createSignal<RotationStatusType | null>(cachedStatus);
  const [timeLeft, setTimeLeft] = createSignal(cachedTimeLeft);
  const [isRotating, setIsRotating] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [initState, setInitState] = createSignal<
    "idle" | "initializing" | "initialized" | "failed"
  >(cachedStatus?.active ? "initialized" : "idle");
  let fetchStatusSequence = 0;

  // Parse proxy URL to extract IP and Port
  const parseProxyUrl = (
    url: string,
  ): { ip: string; port: string; protocol: string } => {
    if (!url) return { ip: "-", port: "-", protocol: "-" };
    try {
      const urlObj = new URL(url);
      return {
        ip: urlObj.hostname,
        port: urlObj.port,
        protocol: urlObj.protocol.replace(":", ""),
      };
    } catch {
      // Fallback for non-standard URLs
      const match = url.match(
        /^(?:https?|socks5):\/\/(?:[^@]+@)?([^:]+):(\d+)$/,
      );
      if (match) {
        return {
          ip: match[1],
          port: match[2],
          protocol: url.startsWith("socks5") ? "socks5" : "http",
        };
      }
      return { ip: "-", port: "-", protocol: "-" };
    }
  };

  // Format seconds to MM:SS
  const formatTime = (seconds: number): string => {
    if (seconds <= 0) return "00:00";
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  // Get progress percentage for the countdown bar
  const getProgressPercent = (): number => {
    const s = status();
    if (!s || s.ttlSeconds === 0) return 0;
    return Math.max(0, Math.min(100, (timeLeft() / s.ttlSeconds) * 100));
  };

  // Get color based on remaining time
  const getTimeColor = (): string => {
    const remaining = timeLeft();
    if (remaining < 60) return "text-red-600 dark:text-red-400";
    if (remaining < 300) return "text-amber-600 dark:text-amber-400";
    return "text-green-600 dark:text-green-400";
  };

  // Get progress bar color
  const getProgressColor = (): string => {
    const percent = getProgressPercent();
    if (percent < 10) return "bg-red-500";
    if (percent < 30) return "bg-amber-500";
    return "bg-green-500";
  };

  // Fetch initial status
  const fetchStatus = async (reason = "unknown") => {
    const seq = ++fetchStatusSequence;
    const startedAt = performance.now();
    console.debug("[RotationStatus] fetchStatus start", {
      seq,
      reason,
      initState: initState(),
      currentTimeLeft: timeLeft(),
      hasStatus: !!status(),
    });

    try {
      const rotationStatus = await getRotationStatus();
      console.debug("[RotationStatus] fetched status", {
        seq,
        reason,
        active: rotationStatus.active,
        currentProxy: rotationStatus.currentProxy,
        expiresInSeconds: rotationStatus.expiresInSeconds,
        ttlSeconds: rotationStatus.ttlSeconds,
        durationMs: Math.round(performance.now() - startedAt),
      });
      setStatus(rotationStatus);
      // Cache the status for instant display on remount
      rotationStore.cacheStatus(rotationStatus);
      if (rotationStatus.active) {
        setTimeLeft(rotationStatus.expiresInSeconds);
        setError(null);
      }
    } catch (e) {
      console.error("Failed to fetch rotation status:", e, {
        seq,
        reason,
        durationMs: Math.round(performance.now() - startedAt),
      });
      setError(String(e));
    }
  };

  // Handle manual rotation
  const handleForceRotate = async () => {
    setIsRotating(true);
    setError(null);
    try {
      const oldProxy = status()?.currentProxy || "";
      const newStatus = await forceRotateProxy();
      console.debug("[RotationStatus] force rotate result", {
        oldProxy,
        newProxy: newStatus.currentProxy,
        changed: oldProxy !== newStatus.currentProxy,
        expiresInSeconds: newStatus.expiresInSeconds,
        ttlSeconds: newStatus.ttlSeconds,
      });
      setStatus(newStatus);
      setTimeLeft(newStatus.expiresInSeconds);
      toastStore.success("Proxy rotated successfully");
    } catch (e) {
      console.error("Failed to rotate proxy:", e);
      setError(String(e));
      toastStore.error("Failed to rotate proxy", String(e));
    } finally {
      setIsRotating(false);
    }
  };

  // Track previous isActive value to avoid unnecessary re-runs
  const [wasActive, setWasActive] = createSignal(false);

  // Setup effects and listeners - use 'on' to explicitly track only props.isActive
  createEffect(
    on(
      () => props.isActive,
      (isActive) => {
        console.debug("[RotationStatus] effect run", {
          isActive,
          wasActive: wasActive(),
          hasCachedStatus: !!rotationStore.cachedState().status,
          cachedActive: rotationStore.cachedState().status?.active,
        });

        if (!isActive) {
          // Don't reset state when inactive - keep cache for instant display when re-activating
          setWasActive(false);
          return;
        }

        // Only setup if transitioning from inactive to active
        if (wasActive()) {
          return;
        }
        setWasActive(true);

        // Check if we have cached status - if so, display immediately without blocking
        const currentCached = rotationStore.cachedState().status;
        if (currentCached?.active && currentCached?.currentProxy) {
          console.debug("[RotationStatus] Using cached status for instant display");
          // Always update status from cache to ensure we have the latest data
          setStatus(currentCached);
          setTimeLeft(currentCached.expiresInSeconds);
          setInitState("initialized");
          setError(null);
        } else {
          // No valid cache, need to initialize
          setInitState("initializing");
        }

        // Setup interval countdown (only when status is active)
        const countdownInterval = setInterval(() => {
          // Chỉ countdown khi đã initialized thành công
          if (initState() !== "initialized") return;

          setTimeLeft((prev) => {
            if (prev <= 0) {
              // Chỉ fetch khi đã có status active — tránh fetch rỗng
              if (status()?.active && status()?.currentProxy) {
                void fetchStatus("countdown<=0");
              }
              return 0;
            }
            return prev - 1;
          });
        }, 1000);

        // Setup Tauri event listeners
        let unlistenUpdated: (() => void) | null = null;
        let unlistenError: (() => void) | null = null;
        let unlistenActive: (() => void) | null = null;
        let unlistenInitialized: (() => void) | null = null;

        // Abort guard để tránh memory leak khi unmount giữa chừng
        let aborted = false;

        const setupListeners = async () => {
          // 1. Đăng ký TẤT CẢ listeners TRƯỚC
          unlistenUpdated = await onRotationProxyUpdated((data) => {
            const previousProxy = status()?.currentProxy || "";
            const eventRemaining = Math.max(
              0,
              Math.floor(data.expiresInSeconds ?? data.ttl),
            );
            console.debug("[RotationStatus] rotation-proxy-updated event", {
              previousProxy,
              newProxy: data.proxy,
              changed: previousProxy !== data.proxy,
              ttl: data.ttl,
              expiresInSeconds: data.expiresInSeconds,
              eventRemaining,
            });
            const newStatus = status()
              ? {
                  ...status()!,
                  currentProxy: data.proxy,
                  realIp: data.realIp ?? status()!.realIp,
                  ttlSeconds: data.ttl,
                  expiresInSeconds: eventRemaining,
                }
              : {
                  active: true,
                  currentProxy: data.proxy,
                  realIp: data.realIp ?? "",
                  ttlSeconds: data.ttl,
                  expiresInSeconds: eventRemaining,
                };
            setStatus(newStatus);
            // Cache the updated status
            rotationStore.cacheStatus(newStatus);
            setTimeLeft(eventRemaining);
            setError(null);
            setInitState("initialized");
          });
          if (aborted) {
            unlistenUpdated();
            return;
          }

          unlistenError = await onRotationProxyError((data) => {
            console.debug("[RotationStatus] rotation-proxy-error event", data);
            setError(data.error);
            toastStore.error("Rotation proxy error", data.error);
          });
          if (aborted) {
            unlistenError();
            unlistenUpdated?.();
            return;
          }

          unlistenActive = await onRotationProxyActive((data) => {
            console.debug("[RotationStatus] rotation-proxy-active event", data);
            if (!data.active) {
              setStatus(null);
              setTimeLeft(0);
              setInitState("idle");
              // Note: We intentionally DON'T clear cache here so that
              // the component can display cached data instantly when re-activated
            }
          });
          if (aborted) {
            unlistenActive();
            unlistenError?.();
            unlistenUpdated?.();
            return;
          }

          // Listen for initialization event from background task
          unlistenInitialized = await onRotationProxyInitialized((data) => {
            console.debug(
              "[RotationStatus] rotation-proxy-initialized event",
              data,
            );
            if (data.success) {
              const newStatus = {
                active: true,
                currentProxy: data.proxy ?? "",
                realIp: data.realIp ?? "",
                ttlSeconds: data.ttl ?? 0,
                expiresInSeconds: data.expiresInSeconds ?? 0,
              };
              setStatus(newStatus);
              // Cache the initialized status
              rotationStore.cacheStatus(newStatus);
              setTimeLeft(data.expiresInSeconds ?? 0);
              setError(null);
              setInitState("initialized");
              toastStore.success("Proxy rotation initialized");
            } else {
              setError(data.error ?? "Initialization failed");
              setInitState("failed");
              toastStore.error(
                "Proxy rotation initialization failed",
                data.error ?? "Unknown error",
              );
            }
          });
          if (aborted) {
            unlistenInitialized();
            unlistenActive?.();
            unlistenError?.();
            unlistenUpdated?.();
            return;
          }

          // 2. CATCH-UP: Sau khi listeners sẵn sàng, kiểm tra trạng thái hiện tại
          //    để xử lý trường hợp event đã phát trước khi listeners đăng ký
          console.debug(
            "[RotationStatus] Listeners ready, performing catch-up fetch",
          );
          try {
            const currentStatus = await getRotationStatus();
            console.debug("[RotationStatus] Catch-up status", currentStatus);

            // Always cache the fetched status for future reference
            rotationStore.cacheStatus(currentStatus);

            // Chỉ cập nhật UI nếu component vẫn đang ở trạng thái initializing
            // (nếu event đã đến qua listener, initState sẽ là "initialized")
            // Hoặc nếu fetched status khác với cached status
            const currentCached = status();
            const hasMeaningfulChange =
              !currentCached ||
              currentCached.currentProxy !== currentStatus.currentProxy ||
              currentCached.active !== currentStatus.active;
            
            if (
              (initState() === "initializing" || hasMeaningfulChange) &&
              currentStatus.active &&
              currentStatus.currentProxy
            ) {
              console.debug(
                "[RotationStatus] Catch-up: found active proxy, updating state",
              );
              setStatus(currentStatus);
              setTimeLeft(currentStatus.expiresInSeconds);
              setError(null);
              setInitState("initialized");
            }
          } catch (e) {
            console.warn("[RotationStatus] Catch-up fetch failed:", e);
            // Không set error — listener vẫn có thể nhận event sau
            // Và cached data (nếu có) vẫn được hiển thị
          }
        };

        void setupListeners();

        onCleanup(() => {
          aborted = true; // Signal cho async setup dừng lại
          clearInterval(countdownInterval);
          unlistenUpdated?.();
          unlistenError?.();
          unlistenActive?.();
          unlistenInitialized?.();
        });
      },
      // Removed { defer: true } to ensure effect runs immediately on mount
      // This fixes the issue where isActive is already true when component mounts
    ),
  );

  const proxy = () => parseProxyUrl(status()?.currentProxy || "");
  const isExpired = () => timeLeft() <= 0 && status()?.active;

  return (
    <div class="rounded-xl border border-brand-200 bg-gradient-to-br from-brand-50 to-white p-4 dark:border-brand-800 dark:from-brand-900/20 dark:to-gray-800/50">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <div class="relative flex h-3 w-3">
            <Show
              when={status()?.active}
              fallback={
                <>
                  <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-gray-400 opacity-75" />
                  <span class="relative inline-flex h-3 w-3 rounded-full bg-gray-500" />
                </>
              }
            >
              <span
                class={`absolute inline-flex h-full w-full animate-ping rounded-full opacity-75 ${isExpired() ? "bg-red-400" : "bg-green-400"}`}
              />
              <span
                class={`relative inline-flex h-3 w-3 rounded-full ${isExpired() ? "bg-red-500" : "bg-green-500"}`}
              />
            </Show>
          </div>
          <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
            Rotation Proxy Status
          </h3>
        </div>
        <button
          class="flex items-center gap-1.5 rounded-lg bg-brand-600 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-brand-700 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-brand-600 dark:hover:bg-brand-500"
          disabled={isRotating() || !status()?.active}
          onClick={handleForceRotate}
          title={
            status()?.active
              ? "Rotate to a new proxy IP"
              : "Start the proxy first to enable rotation"
          }
        >
          <Show when={isRotating()}>
            <svg
              class="h-3.5 w-3.5 animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              />
              <path
                class="opacity-75"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                fill="currentColor"
              />
            </svg>
          </Show>
          <Show when={!isRotating()}>
            <svg
              class="h-3.5 w-3.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
              />
            </svg>
          </Show>
          {isRotating() ? "Rotating..." : "Rotate Now"}
        </button>
      </div>

      <Show when={status()?.active}>
        {/* Provider Info */}
        <Show when={status()?.providerName}>
          <div class="mb-3 flex items-center gap-2 rounded-lg bg-brand-100/50 px-3 py-2 dark:bg-brand-900/30">
            <svg
              class="h-4 w-4 text-brand-600 dark:text-brand-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
              />
            </svg>
            <span class="text-sm font-medium text-brand-800 dark:text-brand-300">
              {status()?.providerName}
            </span>
          </div>
        </Show>
        <div class="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
          {/* Current IP - ô nổi bật nhất */}
          <div class="rounded-lg bg-blue-50 p-2.5 shadow-sm dark:bg-blue-900/20">
            <p class="text-xs text-blue-600 dark:text-blue-400">Current IP</p>
            <p class="mt-0.5 font-mono text-sm font-semibold text-blue-700 dark:text-blue-300">
              {status()?.realIp || proxy().ip || "-"}
            </p>
          </div>
          {/* Proxy Server */}
          <div class="rounded-lg bg-white p-2.5 shadow-sm dark:bg-gray-800">
            <p class="text-xs text-gray-500 dark:text-gray-400">Proxy Server</p>
            <p class="mt-0.5 font-mono text-sm font-medium text-gray-900 dark:text-gray-100">
              {proxy().ip}:{proxy().port}
            </p>
          </div>
          {/* Protocol */}
          <div class="rounded-lg bg-white p-2.5 shadow-sm dark:bg-gray-800">
            <p class="text-xs text-gray-500 dark:text-gray-400">Protocol</p>
            <p class="mt-0.5 font-mono text-sm font-medium text-gray-900 dark:text-gray-100">
              {proxy().protocol.toUpperCase()}
            </p>
          </div>
          {/* TTL Info */}
          <div class="rounded-lg bg-white p-2.5 shadow-sm dark:bg-gray-800">
            <p class="text-xs text-gray-500 dark:text-gray-400">TTL</p>
            <p class="mt-0.5 font-mono text-sm font-medium text-gray-900 dark:text-gray-100">
              {status()?.ttlSeconds ? `${status()!.ttlSeconds}s` : "-"}
            </p>
          </div>
        </div>

        <div class="mt-3">
          <div class="flex items-center justify-between">
            <span class="text-xs text-gray-500 dark:text-gray-400">
              Time Remaining
            </span>
            <span class={`text-sm font-mono font-semibold ${getTimeColor()}`}>
              {formatTime(timeLeft())}
            </span>
          </div>
          <div class="mt-1.5 h-2 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
            <div
              class={`h-full transition-all duration-1000 ease-linear ${getProgressColor()}`}
              style={{ width: `${getProgressPercent()}%` }}
            />
          </div>
          <Show when={isExpired()}>
            <p class="mt-1 text-xs text-red-600 dark:text-red-400">
              ⚠️ Proxy expired. Fetching new proxy...
            </p>
          </Show>
        </div>
      </Show>

      <Show when={!status()?.active && !error()}>
        <div class="mt-4 rounded-lg bg-amber-50 p-3 dark:bg-amber-900/20">
          <Show
            when={initState() === "initializing"}
            fallback={
              <p class="text-sm text-amber-800 dark:text-amber-200">
                <span class="font-semibold">Proxy not started.</span> Please
                click <span class="font-semibold">"Start Proxy"</span> button
                first to initialize the rotation proxy.
              </p>
            }
          >
            <div class="flex items-center gap-2">
              <svg
                class="h-4 w-4 animate-spin text-amber-600 dark:text-amber-400"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                />
                <path
                  class="opacity-75"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  fill="currentColor"
                />
              </svg>
              <p class="text-sm text-amber-800 dark:text-amber-200">
                <span class="font-semibold">
                  Initializing proxy rotation...
                </span>
                <span class="text-xs text-amber-700 dark:text-amber-300 block">
                  Fetching initial proxy from provider
                </span>
              </p>
            </div>
          </Show>
        </div>
      </Show>

      <Show when={error()}>
        <div class="mt-4 rounded-lg bg-red-50 p-3 dark:bg-red-900/20">
          <p class="text-sm text-red-800 dark:text-red-200">⚠️ {error()}</p>
        </div>
      </Show>
    </div>
  );
}
