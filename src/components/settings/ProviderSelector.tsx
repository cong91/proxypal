import {
  createEffect,
  createSignal,
  For,
  on,
  onMount,
  Show,
  untrack,
} from "solid-js";
import { useI18n } from "../../i18n";
import {
  getAvailableRotationProviders,
  getProviderMetadata,
  type ProviderInfo,
  type ProviderMetadata,
  type RotationProxySettings,
} from "../../lib/tauri/proxy";
import { toastStore } from "../../stores/toast";

interface ProviderSelectorProps {
  /** Current rotation URL value (for two-way sync) */
  rotationUrl: string;
  /** Callback when URL should be updated (from form changes) */
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
  const [hasLoadedMetadata, setHasLoadedMetadata] = createSignal<
    Record<string, boolean>
  >({});

  // Metadata for selected provider
  const [metadata, setMetadata] = createSignal<ProviderMetadata | null>(null);
  const [loadingMetadata, setLoadingMetadata] = createSignal(false);

  // Form state (local state for form fields)
  const [apiKey, setApiKey] = createSignal("");
  const [networkType, setNetworkType] = createSignal("random");
  const [locationFilter, setLocationFilter] = createSignal("0");

  // Track if we're currently syncing to avoid loops
  const [isSyncingFromUrl, setIsSyncingFromUrl] = createSignal(false);

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

  // Load metadata when provider changes
  const loadMetadata = async (providerId: string) => {
    if (!providerId) return;

    // Skip if already loaded for this provider
    if (hasLoadedMetadata()[providerId]) return;

    setLoadingMetadata(true);
    try {
      const meta = await getProviderMetadata(providerId);
      setMetadata(meta);
      setHasLoadedMetadata((prev) => ({ ...prev, [providerId]: true }));
    } catch (error) {
      console.error("Failed to load provider metadata:", error);
      toastStore.error("Failed to load provider metadata", String(error));
      setMetadata(null);
    } finally {
      setLoadingMetadata(false);
    }
  };

  // Initial load - only once on mount
  onMount(() => {
    if (!hasLoadedProviders()) {
      setHasLoadedProviders(true);
      void loadProviders();
    }
  });

  // Sync FROM URL -> Form (when URL changes externally)
  // Use 'on' with defer to only run when rotationUrl actually changes
  createEffect(
    on(
      () => props.rotationUrl,
      (url) => {
        const parsed = parseRotationUrl(url);

        if (parsed) {
          setIsSyncingFromUrl(true);

          // Update provider selection
          if (
            parsed.providerId &&
            parsed.providerId !== untrack(selectedProviderId)
          ) {
            setSelectedProviderId(parsed.providerId);
            void loadMetadata(parsed.providerId);
          }

          // Update form fields
          setApiKey(parsed.apiKey);
          setNetworkType(parsed.networkType || "random");
          setLocationFilter(parsed.locationFilter || "0");

          // Notify parent of parsed settings
          props.onSettingsChange?.(parsed);

          // Reset syncing flag after a tick
          setTimeout(() => setIsSyncingFromUrl(false), 0);
        } else {
          // URL is not a valid rotation URL
          props.onSettingsChange?.(undefined);
        }
      },
      { defer: true },
    ),
  );

  // Load metadata when provider selection changes (only if not syncing from URL)
  // Use 'on' to explicitly track only selectedProviderId
  createEffect(
    on(
      selectedProviderId,
      (providerId) => {
        if (providerId && !untrack(isSyncingFromUrl)) {
          void loadMetadata(providerId);
        }
      },
      { defer: true },
    ),
  );

  // Sync FROM Form -> URL (when form fields change)
  const syncToUrl = () => {
    if (untrack(isSyncingFromUrl)) return;

    const providerId = untrack(selectedProviderId);
    if (!providerId) return;

    const settings: RotationProxySettings = {
      providerId,
      apiKey: untrack(apiKey),
      networkType: untrack(networkType),
      locationFilter: untrack(locationFilter),
    };

    // Build and update URL
    const domain = getProviderDomain(providerId);
    const newUrl = buildRotationUrl(domain, settings);

    // Only update if URL actually changed (use untrack to avoid reactivity issues)
    if (newUrl !== untrack(() => props.rotationUrl)) {
      props.onUrlChange(newUrl);
    }

    // Notify parent of settings change
    props.onSettingsChange?.(settings);
  };

  // Handlers that sync to URL
  const handleProviderChange = (providerId: string) => {
    setSelectedProviderId(providerId);
    // Reset to defaults when provider changes
    setApiKey("");
    setNetworkType("random");
    setLocationFilter("0");
    // Sync will happen in effect
    setTimeout(syncToUrl, 0);
  };

  const handleApiKeyChange = (value: string) => {
    setApiKey(value);
    syncToUrl();
  };

  const handleNetworkTypeChange = (value: string) => {
    setNetworkType(value);
    syncToUrl();
  };

  const handleLocationFilterChange = (value: string) => {
    setLocationFilter(value);
    syncToUrl();
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
          Rotation Proxy Configuration
        </h3>
      </div>

      <p class="text-xs text-gray-500 dark:text-gray-400">
        Configure your rotation proxy settings below. The URL above will be
        automatically updated.
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

      <Show when={loadingMetadata()}>
        <div class="flex items-center justify-center py-4">
          <div class="h-5 w-5 animate-spin rounded-full border-2 border-brand-500 border-t-transparent" />
        </div>
      </Show>

      <Show when={metadata() && !loadingMetadata()}>
        <div class="space-y-4">
          {/* API Key */}
          <Show when={metadata()?.requiredFields.includes("apiKey")}>
            <label class="block">
              <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                API Key
              </span>
              <input
                class="transition-smooth mt-1 block w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm focus:border-transparent focus:ring-2 focus:ring-brand-500 dark:border-gray-600 dark:bg-gray-900"
                onInput={(e) => handleApiKeyChange(e.currentTarget.value)}
                placeholder="Enter your API key"
                type="password"
                value={apiKey()}
              />
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                Your provider API key for authentication
              </p>
            </label>
          </Show>

          {/* Network Type */}
          <Show when={(metadata()?.networkOptions?.length ?? 0) > 0}>
            <label class="block">
              <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                Network Type
              </span>
              <select
                class="transition-smooth mt-1 block w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm focus:border-transparent focus:ring-2 focus:ring-brand-500 dark:border-gray-600 dark:bg-gray-900"
                onChange={(e) => handleNetworkTypeChange(e.currentTarget.value)}
                value={networkType()}
              >
                <For each={metadata()?.networkOptions ?? []}>
                  {(option) => (
                    <option value={option.value}>{option.label}</option>
                  )}
                </For>
              </select>
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                Select preferred network provider
              </p>
            </label>
          </Show>

          {/* Location Filter */}
          <Show when={(metadata()?.locationOptions?.length ?? 0) > 0}>
            <label class="block">
              <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                Location
              </span>
              <select
                class="transition-smooth mt-1 block w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm focus:border-transparent focus:ring-2 focus:ring-brand-500 dark:border-gray-600 dark:bg-gray-900"
                onChange={(e) =>
                  handleLocationFilterChange(e.currentTarget.value)
                }
                value={locationFilter()}
              >
                <For each={metadata()?.locationOptions ?? []}>
                  {(option) => (
                    <option value={option.value}>{option.label}</option>
                  )}
                </For>
              </select>
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                Select preferred location
              </p>
            </label>
          </Show>
        </div>
      </Show>

      {/* Info note - no separate save button needed */}
      <div class="rounded-lg bg-blue-50 px-3 py-2 dark:bg-blue-900/20">
        <p class="text-xs text-blue-600 dark:text-blue-400">
          <span class="font-medium">💡 Tip:</span> Changes are automatically
          synced to the URL above. Use the main "Save Settings" button to save
          all proxy configuration.
        </p>
      </div>
    </div>
  );
}
