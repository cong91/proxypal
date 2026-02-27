# Tasks: Proxy Rotation

**Input**: Design documents from `/specs/proxy-rotation/`
**Prerequisites**: plan.md (required), integration guide spec (required)

## Phase 1: Data Types & Structures

**Purpose**: Define Rust types for rotation proxy data

- [x] T001 [P] Add `RotationProxyResponse`, `CachedProxy`, `RotationConfig` structs to `src-tauri/src/types/proxy.rs`
  - `RotationProxyResponse`: mirrors JSON from provider API (`status`, `message`, `proxyhttp`, `proxysocks5`, `Token expiration date`)
  - `CachedProxy`: `http_proxy: String`, `socks5_proxy: String`, `expires_at: Instant`, `ttl_seconds: u64`
  - `RotationConfig`: `api_url: String`, `api_key: String`, `nhamang: String`, `tinhthanh: String` (parsed from `rotation://` URL)
  - Derive `Debug, Clone, Serialize, Deserialize` as appropriate
  - → Verify: `cargo check` passes

---

## Phase 2: Core Rotation Provider

**Purpose**: Implement the rotation proxy provider with cache + fetch logic

- [x] T002 Implement `RotationProxyProvider` struct in `src-tauri/src/proxy/mod.rs`
  - Fields: `config: RotationConfig`, `cache: Arc<tokio::sync::RwLock<Option<CachedProxy>>>`, `client: reqwest::Client`
  - Constructor `fn new(rotation_url: &str) -> Result<Self, String>` that parses `rotation://host?key=X&nhamang=Y&tinhthanh=Z`
  - → Verify: `cargo check` passes

- [x] T003 Implement `fn parse_rotation_url(url: &str) -> Result<RotationConfig, String>` in `src-tauri/src/proxy/mod.rs`
  - Parse scheme `rotation://`, extract host as API base, query params as config fields
  - Build full API URL: `https://{host}/api/get.php?key={key}&nhamang={nhamang}&tinhthanh={tinhthanh}`
  - → Verify: unit test with sample rotation URL passes

- [x] T004 Implement `async fn fetch_proxy(&self) -> Result<CachedProxy, String>` in `src-tauri/src/proxy/mod.rs`
  - Send GET to provider API URL
  - Deserialize `RotationProxyResponse`
  - Parse TTL from `message` field using regex `(\d+)s` pattern
  - Default TTL = 1800s if parsing fails
  - Subtract 30s safety margin from TTL
  - Return `CachedProxy` with computed `expires_at`
  - Retry up to 3 times with exponential backoff (2s, 4s, 8s) on fetch failure
  - → Verify: unit test with mocked response passes

- [x] T005 Implement `async fn get_or_refresh(&self) -> Result<CachedProxy, String>` in `src-tauri/src/proxy/mod.rs`
  - Read lock: check if cache is valid (exists + not expired)
  - If valid: return cached proxy
  - If expired/missing: acquire write lock, double-check, call `fetch_proxy()`, update cache
  - → Verify: unit test for cache hit + cache miss scenarios

- [x] T006 Implement `fn resolve_proxy_url(cached: &CachedProxy) -> String` helper in `src-tauri/src/proxy/mod.rs`
  - Convert provider format `host:port:user:pass` to standard URL `http://user:pass@host:port`
  - Support both HTTP and SOCKS5 proxy strings
  - Prefer SOCKS5 if available, fallback to HTTP
  - → Verify: unit test with various proxy string formats

**Checkpoint**: Core rotation logic complete, independently testable

---

## Phase 3: State Integration

**Purpose**: Wire rotation provider into Tauri app state

- [x] T007 Add `rotation_provider: Mutex<Option<Arc<RotationProxyProvider>>>` field to `AppState` in `src-tauri/src/state.rs`
  - Update `Default` impl to initialize as `Mutex::new(None)`
  - → Verify: `cargo check` passes

---

## Phase 4: Proxy Command Integration

**Purpose**: Modify proxy start/stop to detect rotation URLs and manage lifecycle

- [x] T008 Add `fn is_rotation_url(url: &str) -> bool` helper in `src-tauri/src/commands/proxy.rs`
  - Check if URL starts with `rotation://`
  - → Verify: trivial unit test

- [x] T009 Modify `start_proxy()` in `src-tauri/src/commands/proxy.rs` to handle rotation URLs
  - After loading config, check `is_rotation_url(&effective_proxy_url)`
  - If rotation: create `RotationProxyProvider`, call `get_or_refresh()` to get initial proxy
  - Use resolved proxy URL in `build_proxy_url_line()` instead of the rotation:// URL
  - Store provider in `AppState.rotation_provider`
  - → Verify: `cargo check` passes; start_proxy with rotation URL resolves to real proxy

- [x] T010 Implement background TTL monitor task in `src-tauri/src/commands/proxy.rs`
  - Spawn `tokio::spawn` task after sidecar starts (only if rotation is active)
  - Poll every 30 seconds: check if cached proxy is about to expire (within 60s)
  - On expiration: call `get_or_refresh()` (triggers new fetch), then update sidecar via Management API
  - Use `PUT http://127.0.0.1:{port}/v0/management/proxy-url` with header `X-Management-Key`
  - If Management API fails: log warning, attempt sidecar restart as fallback
  - Monitor task stops when `log_watcher_running` is set to false (reuse existing signal)
  - → Verify: `cargo check` passes

- [x] T011 Modify `stop_proxy()` in `src-tauri/src/commands/proxy.rs`
  - Clear `AppState.rotation_provider` to `None` when proxy stops
  - TTL monitor task will stop naturally via the existing `log_watcher_running` flag
  - → Verify: stopping proxy clears rotation state

- [x] T012 Add `get_rotation_status` Tauri command in `src-tauri/src/commands/proxy.rs`
  - Return JSON: `{ active: bool, current_proxy: String, expires_in_seconds: u64, ttl_seconds: u64 }`
  - If no rotation active, return `{ active: false, ... }`
  - → Verify: `cargo check` passes

- [x] T013 Register `get_rotation_status` in `invoke_handler` in `src-tauri/src/lib.rs`
  - Add `commands::proxy::get_rotation_status` to the handler list
  - → Verify: `cargo check` passes

**Checkpoint**: Backend fully functional - rotation proxy starts, monitors TTL, rotates automatically

---

## Phase 5: Frontend UI Updates

**Purpose**: Add rotation status visibility and help text to Settings UI

- [x] T014 [P] Add `getRotationStatus()` Tauri binding in `src/lib/tauri/proxy.ts`
  - Call `invoke("get_rotation_status")`
  - Type the response: `{ active: boolean, currentProxy: string, expiresInSeconds: number, ttlSeconds: number }`
  - → Verify: TypeScript compiles

- [x] T015 Update `ProxySettings.tsx` in `src/components/settings/ProxySettings.tsx`
  - Add help text under proxy URL input: "Supports rotation:// URLs for automatic IP rotation"
  - Add a rotation status badge (green dot + "Rotation active" or "Next rotation in Xs") when rotation is active
  - Poll `getRotationStatus()` every 10s when proxy is running and rotation is active
  - → Verify: UI shows rotation status when using rotation:// URL

**Checkpoint**: Feature complete end-to-end

---

## Phase 6: Error Handling & Edge Cases

**Purpose**: Harden error handling for production use

- [x] T016 Add force-rotation on connection error in `src-tauri/src/proxy/mod.rs`
  - Add `async fn force_rotate(&self) -> Result<CachedProxy, String>` that invalidates cache and fetches new proxy
  - → Verify: unit test confirms cache is cleared and new proxy fetched

- [x] T017 Add `force_rotate_proxy` Tauri command in `src-tauri/src/commands/proxy.rs`
  - Allows frontend to manually trigger rotation (e.g., user clicks "Rotate Now" button)
  - Calls `provider.force_rotate()` then updates sidecar via Management API
  - Register in `invoke_handler` in `src-tauri/src/lib.rs`
  - → Verify: `cargo check` passes

- [x] T018 Emit Tauri events for rotation lifecycle in proxy commands
  - `rotation-proxy-updated`: when a new proxy IP is activated (include new IP, TTL)
  - `rotation-proxy-error`: when fetch fails after all retries (include error message)
  - `rotation-proxy-fallback`: when falling back to direct connection
  - → Verify: events are emitted correctly during rotation

---

## Phase 7: Polish

**Purpose**: Documentation and cleanup

- [x] T019 [P] Add doc comments to all public functions in `src-tauri/src/proxy/mod.rs`
  - Document each public method with `///` comments
  - Add module-level `//!` documentation explaining the rotation mechanism
  - → Verify: `cargo doc` generates clean documentation

- [ ] T020 Update README or add inline config documentation
  - Document the `rotation://` URL format: `rotation://proxyxoay.shop?key=XXX&nhamang=random&tinhthanh=0`
  - Document supported query parameters
  - → Verify: documentation is clear and complete

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** (Types): No dependencies - start immediately
- **Phase 2** (Provider): Depends on Phase 1 types
- **Phase 3** (State): Depends on Phase 2 struct definition
- **Phase 4** (Commands): Depends on Phase 2 + Phase 3
- **Phase 5** (Frontend): Depends on Phase 4 (T012, T013 for Tauri command)
- **Phase 6** (Error handling): Depends on Phase 2 + Phase 4
- **Phase 7** (Polish): Depends on all above

### Parallel Opportunities

- T001 types can run in parallel with any non-dependent prep work
- T014 frontend binding can start as soon as T012 is defined (interface contract)
- T019 docs and T020 README can run in parallel with Phase 6
- Phase 5 and Phase 6 can run in parallel after Phase 4

### Critical Path

```
T001 → T002 → T003 → T004 → T005 → T006 → T007 → T009 → T010 → T013
```

---

## Notes

- No new crate dependencies needed - `reqwest`, `tokio`, `regex`, `serde` are already in Cargo.toml
- The `rotation://` scheme is a custom URL scheme - not IANA registered. It's an internal convention from the reference project CLIProxyAPIPlus
- If the provider API changes format, only `fetch_proxy()` and `parse_ttl()` need updating
- The TTL monitor reuses the existing `log_watcher_running` AtomicBool signal for clean shutdown
