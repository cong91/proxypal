import {
  createEffect,
  createSignal,
  For,
  on,
  onMount,
  untrack,
} from "solid-js";

// Deep equality check for rotation settings
function rotationSettingsEqual(
  a: RotationProxySettings | undefined,
  b: RotationProxySettings | undefined,
): boolean {
  if (a === b) return true;
  if (!a || !b) return false;
  return (
    a.providerId === b.providerId &&
    a.apiKey === b.apiKey &&
    a.networkType === b.networkType &&
    a.locationFilter === b.locationFilter
  );
}
import { useI18n } from "../../i18n";
import {
  getAvailableRotationProviders,
  type ProviderInfo,
  type RotationProxySettings,
} from "../../lib/tauri/proxy";
import { toastStore } from "../../stores/toast";

interface ProviderSelectorProps {
  /** Current rotation URL value (for parsing and two-way sync) */
  rotationUrl: string;
  /** Initial settings from config (used to fallback if URL isn't fully set) */
  initialSettings?: RotationProxySettings | null;
  /** Callback when URL should be updated (from provider change) */
  onUrlChange: (url: string) => void;
  /** Callback when settings are parsed from URL (for config update) */
  onSettingsChange?: (settings: RotationProxySettings | undefined) => void;
}

// Provider domain mapping for auto-detection
const PROVIDER_DOMAINS: Record<string, string> = {
  "proxyxoay.shop": "proxy_vn",
  "proxy.vn": "proxy_vn",
};

/**
 * Parse rotation:// URL to extract settings
 * Returns undefined if URL is invalid or not a rotation URL
 */
function parseRotationUrl(url: string): RotationProxySettings | undefined {
  if (!url.startsWith("rotation://")) {
    return undefined;
  }

  try {
    // Remove rotation:// prefix and parse
    const urlPart = url.slice("rotation://".length);
    const [domainPart, queryPart] = urlPart.split("?");
    const domain = domainPart.trim();

    // Parse query parameters
    const params = new URLSearchParams(queryPart || "");
    const apiKey = params.get("key") || "";
    const networkType = params.get("nhamang") || "random";
    const locationFilter = params.get("tinhthanh") || "0";

    // Detect provider from domain
    let providerId = "proxy_vn"; // default
    for (const [domainPattern, pid] of Object.entries(PROVIDER_DOMAINS)) {
      if (domain.includes(domainPattern)) {
        providerId = pid;
        break;
      }
    }

    return {
      providerId,
      apiKey,
      networkType,
      locationFilter,
    };
  } catch {
    return undefined;
  }
}

/**
 * Build rotation:// URL from settings
 */
function buildRotationUrl(
  domain: string,
  settings: RotationProxySettings,
): string {
  const params = new URLSearchParams();
  if (settings.apiKey) params.set("key", settings.apiKey);
  if (settings.networkType) params.set("nhamang", settings.networkType);
  if (settings.locationFilter) params.set("tinhthanh", settings.locationFilter);

  const queryString = params.toString();
  return `rotation://${domain}${queryString ? "?" + queryString : ""}`;
}

/**
 * Get default domain for a provider
 */
function getProviderDomain(providerId: string): string {
  switch (providerId) {
    case "proxy_vn":
      return "proxyxoay.shop";
    default:
      return "proxyxoay.shop";
  }
}

export function ProviderSelector(props: ProviderSelectorProps) {
  void useI18n; // i18n will be used when translations are added

  // Provider list
  const [providers, setProviders] = createSignal<ProviderInfo[]>([]);
  const [selectedProviderId, setSelectedProviderId] = createSignal<string>("");
  const [hasLoadedProviders, setHasLoadedProviders] = createSignal(false);

  // Track last notified settings to avoid redundant callbacks
  const [lastNotifiedSettings, setLastNotifiedSettings] =
    createSignal<RotationProxySettings | undefined>(undefined);

  // Load providers on mount
  const loadProviders = async () => {
    try {
      const available = await getAvailableRotationProviders();
      setProviders(available);
    } catch (error) {
      console.error("Failed to load providers:", error);
      toastStore.error("Failed to load rotation providers", String(error));
    }
  };

  // Helper to get effective settings (URL first, then initialSettings from config)
  const getEffectiveSettings = (): RotationProxySettings | undefined => {
    const urlSettings = parseRotationUrl(props.rotationUrl);
    if (urlSettings) {
      return urlSettings;
    }
    if (props.initialSettings) {
      return props.initialSettings;
    }
    return undefined;
  };

  // Initial load - only once on mount
  onMount(() => {
    if (!hasLoadedProviders()) {
      setHasLoadedProviders(true);
      void loadProviders();
    }

    const settings = getEffectiveSettings();
    if (settings) {
      if (settings.providerId) {
        setSelectedProviderId(settings.providerId);
      }
      setLastNotifiedSettings(settings);
      props.onSettingsChange?.(settings);
    }
  });

  // Sync FROM URL -> Component State (when URL changes externally)
  createEffect(
    on(
      () => props.rotationUrl,
      (url) => {
        const parsed = parseRotationUrl(url);

        if (parsed) {
          if (
            parsed.providerId &&
            parsed.providerId !== untrack(selectedProviderId)
          ) {
            setSelectedProviderId(parsed.providerId);
          }

          const lastNotified = untrack(lastNotifiedSettings);
          if (!rotationSettingsEqual(parsed, lastNotified)) {
            setLastNotifiedSettings(parsed);
            props.onSettingsChange?.(parsed);
          }
        } else {
          const settings = props.initialSettings;
          if (settings) {
            if (
              settings.providerId &&
              settings.providerId !== untrack(selectedProviderId)
            ) {
              setSelectedProviderId(settings.providerId);
            }

            const lastNotified = untrack(lastNotifiedSettings);
            if (!rotationSettingsEqual(settings, lastNotified)) {
              setLastNotifiedSettings(settings);
              props.onSettingsChange?.(settings);
            }
          } else {
            const lastNotified = untrack(lastNotifiedSettings);
            if (lastNotified !== undefined) {
              setLastNotifiedSettings(undefined);
              props.onSettingsChange?.(undefined);
            }
          }
        }
      },
      { defer: true },
    ),
  );

  const handleProviderChange = (providerId: string) => {
    setSelectedProviderId(providerId);

    // Get current parsed settings to keep query parameters if they exist
    const parsed = parseRotationUrl(props.rotationUrl);
    const domain = getProviderDomain(providerId);

    const settings: RotationProxySettings = {
      providerId,
      apiKey: parsed?.apiKey || "",
      networkType: parsed?.networkType || "random",
      locationFilter: parsed?.locationFilter || "0",
    };

    const newUrl = buildRotationUrl(domain, settings);
    
    if (newUrl !== props.rotationUrl) {
      props.onUrlChange(newUrl);
    }
    
    const lastNotified = untrack(lastNotifiedSettings);
    if (!rotationSettingsEqual(settings, lastNotified)) {
      setLastNotifiedSettings(settings);
      props.onSettingsChange?.(settings);
    }
  };

  return (
    <div class="space-y-4 rounded-xl border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-gray-800/50">
      <div class="flex items-center gap-2">
        <svg
          class="h-4 w-4 text-brand-500"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M13 10V3L4 14h7v7l9-11h-7z"
          />
        </svg>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-white">
          Rotation Proxy Provider
        </h3>
      </div>

      <p class="text-xs text-gray-500 dark:text-gray-400">
        Query parameters in your Upstream Proxy URL will be automatically parsed to configure your connection.
      </p>

      {/* Provider Select */}
      <label class="block">
        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
          Provider
        </span>
        <select
          class="transition-smooth mt-1 block w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm focus:border-transparent focus:ring-2 focus:ring-brand-500 dark:border-gray-600 dark:bg-gray-900"
          disabled={providers().length === 0}
          onChange={(e) => handleProviderChange(e.currentTarget.value)}
          value={selectedProviderId()}
        >
          <For each={providers()}>
            {(provider) => <option value={provider.id}>{provider.name}</option>}
          </For>
        </select>
        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Select your rotation proxy provider
        </p>
      </label>
    </div>
  );
}
