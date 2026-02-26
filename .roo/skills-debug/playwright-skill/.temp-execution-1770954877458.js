const { chromium } = require('playwright');

const TARGET_URL = 'http://localhost:3000';
// Các routes chính xác trong dự án
const ROUTES = [
  '/',                    // Home
  '/auth/signin',         // Login
  '/auth/signup',         // Register  
  '/classroom',
  '/auth/forgot-password',
  '/settings'
];
const VIEWPORTS = [
  { name: 'Desktop', width: 1280, height: 800 },
  { name: 'Mobile', width: 375, height: 667 }
];

(async () => {
  const browser = await chromium.launch({ headless: false });
  const results = [];

  for (const route of ROUTES) {
    console.log(`\n🧪 Testing route: ${route}`);
    
    for (const viewport of VIEWPORTS) {
      const context = await browser.newContext({
        viewport: { width: viewport.width, height: viewport.height }
      });
      const page = await context.newPage();
      
      // Collect console errors
      const consoleErrors = [];
      page.on('console', msg => {
        if (msg.type() === 'error') {
          consoleErrors.push(msg.text());
        }
      });

      // Collect page errors
      const pageErrors = [];
      page.on('pageerror', error => {
        pageErrors.push(error.message);
      });

      // Collect failed CSS resources
      const cssLoadErrors = [];
      page.on('response', response => {
        const url = response.url();
        const status = response.status();
        if (url.includes('.css') && (status >= 400)) {
          cssLoadErrors.push({ url, status });
        }
      });

      try {
        // Navigate with hard reload (no cache)
        const response = await page.goto(`${TARGET_URL}${route}`, {
          waitUntil: 'networkidle',
          timeout: 15000
        });
        
        // Check if page loaded successfully
        if (!response || response.status() >= 400) {
          throw new Error(`Page returned status ${response ? response.status() : 'unknown'}`);
        }
        
        // Hard reload to verify no-cache behavior
        await page.reload({ waitUntil: 'networkidle' });

        await page.waitForTimeout(2000);

        const title = await page.title();

        // Check Tailwind classes
        const tailwindElements = await page.evaluate(() => {
          const allElements = document.querySelectorAll('*');
          let count = 0;
          const patterns = ['flex', 'grid', 'block', 'container', 'mx-auto', 'px-', 'py-', 'bg-', 'text-', 'font-'];
          allElements.forEach(el => {
            const classes = el.className;
            if (typeof classes === 'string') {
              patterns.forEach(p => { if (classes.includes(p)) count++; });
            }
          });
          return count;
        });

        // Check computed styles
        const computedStyles = await page.evaluate(() => {
          const body = document.body;
          const computed = window.getComputedStyle(body);
          return {
            fontFamily: computed.fontFamily,
            backgroundColor: computed.backgroundColor,
            color: computed.color
          };
        });

        // Get CSS files
        const cssFiles = await page.evaluate(() => {
          return Array.from(document.querySelectorAll('link[rel="stylesheet"]'))
            .map(link => link.href);
        });

        // Get first stylesheet (should be app/layout.css)
        const firstStylesheet = cssFiles[0] || 'None';
        const tailwindFirst = firstStylesheet.includes('app/layout.css');

        // Screenshot
        const safeRoute = route.replace(/\//g, '-').replace(/^-/, 'root');
        const screenshotPath = `/tmp/final-${safeRoute}-${viewport.name.toLowerCase()}.png`;
        await page.screenshot({ path: screenshotPath, fullPage: true });

        // Determine status
        const hasCSSErrors = cssLoadErrors.length > 0;
        const hasConsoleCSSErrors = consoleErrors.some(e => 
          e.includes('CSS') || e.includes('stylesheet') || (e.includes('404') && e.includes('.css'))
        );
        const tailwindWorking = tailwindElements > 0;
        
        const status = (!hasCSSErrors && !hasConsoleCSSErrors && tailwindWorking) ? 'PASS' : 'FAIL';

        results.push({
          route,
          viewport: viewport.name,
          title,
          httpStatus: response.status(),
          tailwindElements,
          tailwindWorking,
          tailwindFirst,
          firstStylesheet: firstStylesheet.split('?')[0], // Remove query params
          computedStyles,
          cssFilesCount: cssFiles.length,
          consoleErrors: consoleErrors.length,
          cssLoadErrors: cssLoadErrors.length,
          screenshot: screenshotPath,
          status
        });

        console.log(`  ✅ ${viewport.name}: ${title}`);
        console.log(`     HTTP: ${response.status()} | Tailwind: ${tailwindElements} | CSS files: ${cssFiles.length}`);
        console.log(`     First CSS: ${tailwindFirst ? '✓ layout.css' : '✗ ' + firstStylesheet.split('/').pop()}`);
        if (consoleErrors.length > 0) console.log(`     Console errors: ${consoleErrors.length}`);
        if (cssLoadErrors.length > 0) console.log(`     CSS load errors: ${cssLoadErrors.length}`);

      } catch (error) {
        results.push({
          route,
          viewport: viewport.name,
          error: error.message,
          status: 'ERROR'
        });
        console.log(`  ❌ ${viewport.name}: ERROR - ${error.message}`);
      }

      await context.close();
    }
  }

  await browser.close();

  // Summary
  console.log('\n' + '='.repeat(70));
  console.log('CSS GLOBAL FIX - FINAL REGRESSION TEST SUMMARY');
  console.log('='.repeat(70));
  
  results.forEach(r => {
    const icon = r.status === 'PASS' ? '✅' : r.status === 'FAIL' ? '❌' : '⚠️';
    console.log(`${icon} ${r.route.padEnd(25)} (${r.viewport.padEnd(7)}): ${r.status}`);
  });

  const passed = results.filter(r => r.status === 'PASS').length;
  const failed = results.filter(r => r.status === 'FAIL').length;
  const errors = results.filter(r => r.status === 'ERROR').length;

  console.log('\n' + '-'.repeat(70));
  console.log(`Total: ${results.length} tests`);
  console.log(`✅ Passed: ${passed}`);
  console.log(`❌ Failed: ${failed}`);
  console.log(`⚠️ Errors: ${errors}`);

  // Detailed JSON output
  console.log('\n' + '='.repeat(70));
  console.log('DETAILED RESULTS');
  console.log('='.repeat(70));
  console.log(JSON.stringify(results, null, 2));

  process.exit(failed > 0 || errors > 0 ? 1 : 0);
})();
