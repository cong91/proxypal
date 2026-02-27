# Hướng dẫn tích hợp Proxy Xoay (Rotation Proxy) vào dự án mới

## Tổng quan
Tính năng proxy xoay cho phép ứng dụng tự động gọi API lấy IP/Port proxy mới từ nhà cung cấp (ví dụ: proxyxoay.shop) khi proxy cũ sắp hết hạn hoặc không còn khả dụng, giúp ứng dụng duy trì kết nối mạng ổn định qua các IP khác nhau để vượt qua rate-limit hoặc IP ban.

Tài liệu này dựa trên template đã được triển khai thành công tại dự án `CLIProxyAPIPlus`.

## 1. Cơ chế hoạt động của Rotation Proxy
1. **Khởi tạo:** Ứng dụng nhận cấu hình proxy dạng URL ảo (VD: `rotation://proxyxoay.shop?key=XXX&nhamang=random&tinhthanh=0`).
2. **Fetch Proxy:** Dùng HTTP Client gọi API của nhà cung cấp (thường là `https://[host]/api/get.php?key=...`) để lấy thông tin proxy (HTTP, SOCKS5) và thời gian sống (TTL).
3. **Cache (Lưu trữ):** Lưu proxy vừa lấy được vào biến toàn cục (kèm theo khoá `sync.RWMutex` trong Go để an toàn đa luồng) và đánh dấu thời gian hết hạn (`ExpiresAt`).
4. **Sử dụng:** Mỗi khi ứng dụng cần gửi request, nó sẽ gọi hàm lấy proxy. Hàm này kiểm tra nếu thời gian hiện tại đã vượt qua `ExpiresAt` thì sẽ tự động gọi lại bước 2 để lấy proxy mới, nếu không thì trả về proxy trong cache.
5. **Gắn vào Client:** Inject proxy string vào cấu hình Transport của HTTP/HTTPS client (hỗ trợ cả HTTP Transport và SOCKS5 Dialer).

## 2. Các thành phần chính cần có (Ví dụ mã nguồn Go)

### 2.1 Cấu trúc dữ liệu (Data Structures)
Cần định nghĩa Struct để nhận JSON response từ API và lưu Cache nội bộ:
```go
// Response từ API của nhà cung cấp proxy
type RotationProxyResponse struct {
	Status             int    `json:"status"`
	Message            string `json:"message"`
	ProxyHTTP          string `json:"proxyhttp"`
	ProxySocks5        string `json:"proxysocks5"`
	TokenExpirationDate string `json:"Token expiration date"`
}

// Cấu trúc lưu trữ proxy tại bộ nhớ nội bộ
type CachedProxy struct {
	HTTPProxy   string
	Socks5Proxy string
	ExpiresAt   time.Time
	TTLSeconds  int
}
```

### 2.2 Đối tượng quản lý Proxy (Provider)
Tạo đối tượng để quản lý việc gọi API và giữ Cache:
```go
type RotationProxyProvider struct {
	apiURL      string
	apiKey      string
	nhamang     string
	tinhthanh   string
	cache       *CachedProxy
	cacheMutex  sync.RWMutex
	client      *http.Client
}
```

### 2.3 Hàm kết nối API & lấy Proxy mới (Fetch)
```go
func (r *RotationProxyProvider) FetchProxy() (*CachedProxy, error) {
	requestURL := fmt.Sprintf("%s?key=%s&nhamang=%s&tinhthanh=%s", r.apiURL, r.apiKey, r.nhamang, r.tinhthanh)
    // Gửi GET request
	resp, err := r.client.Get(requestURL)
    // ... Kiểm tra mã lỗi ...
    
	var proxyResp RotationProxyResponse
	if err := json.NewDecoder(resp.Body).Decode(&proxyResp); err != nil {
		return nil, err
	}

    // Tách thời gian sống (TTL) từ thông báo, ví dụ "proxy nay se die sau 1777s"
	ttlSeconds := r.parseTTL(proxyResp.Message) 

	cached := &CachedProxy{
		HTTPProxy:   proxyResp.ProxyHTTP,
		Socks5Proxy: proxyResp.ProxySocks5,
		ExpiresAt:   time.Now().Add(time.Duration(ttlSeconds) * time.Second),
		TTLSeconds:  ttlSeconds,
	}
	return cached, nil
}
```

### 2.4 Hàm lấy Proxy thông minh (Check Expiration & Rotate)
```go
func (r *RotationProxyProvider) GetCachedProxy() (*CachedProxy, error) {
	r.cacheMutex.RLock()
	cached := r.cache
	r.cacheMutex.RUnlock()

	// Nếu chưa có cache hoặc đã quá giờ ExpiresAt
	if cached == nil || time.Now().After(cached.ExpiresAt) {
		r.cacheMutex.Lock()
		defer r.cacheMutex.Unlock()

		// Double-check pattern để tránh gọi API nhiều lần khi có nhiều luồng cùng kẹt ở Lock
		if r.cache == nil || time.Now().After(r.cache.ExpiresAt) {
			newCache, err := r.FetchProxy()
			if err != nil {
				return nil, err
			}
			r.cache = newCache
			cached = newCache
		}
	}
	return cached, nil
}
```

### 2.5 Hàm tích hợp (Inject) vào HTTP Client của ứng dụng
Sau khi lấy được Proxy String (định dạng `host:port:user:pass`), ta cần gắn nó vào HTTP client:

```go
func SetRotationProxy(cfgURL string, httpClient *http.Client) *http.Client {
    // 1. Lấy provider từ cfgURL (rotation://...)
	provider, err := NewRotationProxyProvider(cfgURL)
    
    // 2. Lấy proxy (sẽ tự động fetch nếu hết hạn)
	cachedProxy, err := provider.GetCachedProxy()

    // 3. Phân tích Proxy String sang cấu trúc URL
	var transport *http.Transport
	if cachedProxy.Socks5Proxy != "" {
		proxyURL, _ := ParseProxyString(cachedProxy.Socks5Proxy, "socks5")
		transport = buildSOCKS5Transport(proxyURL) // Sử dụng proxy.SOCKS5 dialer
	} else if cachedProxy.HTTPProxy != "" {
		proxyURL, _ := ParseProxyString(cachedProxy.HTTPProxy, "http")
		transport = &http.Transport{Proxy: http.ProxyURL(proxyURL)} // HTTP Proxy chuẩn
	}

    // 4. Gắn transport vào HTTP Client hiện tại
	if transport != nil {
		httpClient.Transport = transport
	}
	return httpClient
}
```

## 3. Các bước triển khai cho một dự án TypeScript/NodeJS (ví dụ TAA)

Nếu dự án dùng **TypeScript/NodeJS**, mô hình tương tự như sau:
1. **Lớp ProxyManager**: Dùng `axios` hoặc `fetch` gọi API lấy IP.
2. **Cơ chế Cache**: Sử dụng class property `expiresAt` (kiểu `Date`) và một `activeProxy` object.
3. **Mỗi request đi ra**: Sử dụng axios interceptor, trước khi request đi, gọi hàm `getProxy()`. Hàm này sẽ kiểm tra `Date.now() > expiresAt`. Nếu hết hạn, nó sẽ dùng mutex/lock (hoặc 1 promise đang pending) để gọi API lấy proxy mới.
4. **Gắn Proxy**: Dùng thư viện `https-proxy-agent` hoặc `socks-proxy-agent` cấu hình `agent` cho request đó.

### Ví dụ mô phỏng NodeJS:
```typescript
import { HttpsProxyAgent } from 'https-proxy-agent';
import axios from 'axios';

class RotationProxyService {
  private activeProxy: string | null = null;
  private expiresAt: number = 0;
  private fetchPromise: Promise<string> | null = null;

  async getAgent(): Promise<HttpsProxyAgent | null> {
    // 1. Kiểm tra hết hạn
    if (Date.now() > this.expiresAt || !this.activeProxy) {
      // Dùng fetchPromise để tránh việc nhiều request cùng lúc gọi hàm lấy Proxy
      if (!this.fetchPromise) {
        this.fetchPromise = this.fetchNewProxy().finally(() => {
          this.fetchPromise = null;
        });
      }
      await this.fetchPromise;
    }
    
    // 2. Khởi tạo Agent
    if (this.activeProxy) {
        // activeProxy định dạng: http://user:pass@host:port
        return new HttpsProxyAgent(this.activeProxy);
    }
    return null;
  }

  private async fetchNewProxy(): Promise<string> {
    const res = await axios.get('https://proxyxoay.shop/api/get.php?key=XXX');
    const data = res.data;
    
    // Tách số giây từ message
    const ttlMatch = data.message.match(/(\d+)s/);
    const ttlSeconds = ttlMatch ? parseInt(ttlMatch[1]) : 1800; // Mặc định 30p

    // Chuyển format proxy
    const [host, port, user, pass] = data.proxyhttp.split(':');
    this.activeProxy = `http://${user}:${pass}@${host}:${port}`;
    this.expiresAt = Date.now() + (ttlSeconds * 1000) - 5000; // Trừ hao 5 giây
    
    return this.activeProxy;
  }
}

// Khi dùng
const proxyManager = new RotationProxyService();
const proxyAgent = await proxyManager.getAgent();
const response = await axios.get('https://api.binance.com/api/v3/ping', {
    httpsAgent: proxyAgent
});
```

Tài liệu này bao quát cách tiếp cận của dự án CLIProxyAPIPlus và có thể dễ dàng chuyển đổi sang dự án viết bằng ngôn ngữ khác.
