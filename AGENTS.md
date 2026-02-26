# ProxyPal

Tauri v2 desktop app — proxy API management with SolidJS frontend and Rust backend.

## Stack

- **Frontend:** SolidJS 1.9 + TypeScript 5.6 + Tailwind CSS 3 + Kobalte UI
- **Backend:** Rust (Tauri v2, 10 plugins: dialog, fs, updater, deep-link, etc.)
- **Build:** Vite 6 + Tauri CLI | **Test:** Vitest 4 | **PM:** pnpm
- **Charts:** ECharts 6 + Chart.js 4 | **i18n:** @solid-primitives/i18n

## Structure

```
src/                        # SolidJS frontend
  components/               # UI components (22+), charts/, ui/
  pages/                    # Dashboard, Analytics, ApiKeys, Settings, etc.
  stores/                   # Reactive stores (app, requests, theme, toast)
  i18n/                     # Internationalization (locale, catalog)
  lib/                      # Utilities (tauri bindings, quotaCache)
src-tauri/src/              # Rust backend
  commands/                 # Tauri commands (proxy, cloudflare, ssh, config)
  types/                    # Shared type definitions (16 modules)
  lib.rs                    # Main library entry
  config.rs, state.rs       # App config and state management
```

## Commands

```bash
pnpm tauri dev              # Dev (frontend + backend)
pnpm tsc --noEmit           # Type check (frontend)
pnpm check:ts               # Fast type check (uses tsgo when available, falls back to tsc)
pnpm check:parallel         # Parallel: type check + lint + format
cd src-tauri && cargo check # Type check (backend)
pnpm test                   # Vitest (10 tests, 3 files)
pnpm build                  # Vite build (frontend only)
```

## Code Style

```tsx
// SolidJS: interface above component, splitProps, `class` not `className`
interface ProviderCardProps {
  name: string;
  provider: Provider;
  connected: number;
  onConnect: (provider: Provider) => Promise<void>;
}
export function ProviderCard(props: ProviderCardProps) {
  const [loading, setLoading] = createSignal(false);
  // ...
}
```

```rust
// Rust: Result<T, String>, State<AppState>, #[serde(rename_all = "camelCase")]
#[tauri::command]
pub fn save_config(state: State<AppState>, config: AppConfig) -> Result<(), String> {
    let mut current_config = state.config.lock().unwrap();
    *current_config = config;
    Ok(())
}
```

## Boundaries

- **Always:** Run `pnpm tsc --noEmit` + `cd src-tauri && cargo check` before done. Preserve Tailwind card/badge patterns.
- **Ask first:** New dependencies. Modifying `AppConfig` schema. Changing CLIProxyAPI lifecycle.
- **Never:** Commit secrets/`.env`. Blocking IO in async Rust without `spawn_blocking`. Edit `dist/` or `target/`.

## Gotchas

- `lib.rs` is 332KB — navigate with LSP, don't read fully
- Vite build warns about chunk size (>500KB) — expected
- Both `bun.lock` and `pnpm-lock.yaml` exist — use pnpm
# AGENTS

<skills_system priority="1">

## Available Skills

<!-- SKILLS_TABLE_START -->
<usage>
When users ask you to perform tasks, check if any of the available skills below can help complete the task more effectively. Skills provide specialized capabilities and domain knowledge.

How to use skills:
- Invoke: Bash("openskills read <skill-name>")
- The skill content will load with detailed instructions on how to complete the task
- Base directory provided in output for resolving bundled resources (references/, scripts/, assets/)

Usage notes:
- Only use skills listed in <available_skills> below
- Do not invoke a skill that is already loaded in your context
- Each skill invocation is stateless
</usage>

<available_skills>

<skill>
<name>ai-engineer</name>
<description>Build production-ready LLM applications, advanced RAG systems, and</description>
<location>project</location>
</skill>

<skill>
<name>anti-reversing-techniques</name>
<description>Understand anti-reversing, obfuscation, and protection techniques encountered during software analysis. Use when analyzing protected binaries, bypassing anti-debugging for authorized analysis, or understanding software protection mechanisms.</description>
<location>project</location>
</skill>

<skill>
<name>api-documenter</name>
<description>Master API documentation with OpenAPI 3.1, AI-powered tools, and</description>
<location>project</location>
</skill>

<skill>
<name>api-patterns</name>
<description>API design principles and decision-making. REST vs GraphQL vs tRPC selection, response formats, versioning, pagination.</description>
<location>project</location>
</skill>

<skill>
<name>api-security-best-practices</name>
<description>Implement secure API design patterns including authentication, authorization, input validation, rate limiting, and protection against common API vulnerabilities</description>
<location>project</location>
</skill>

<skill>
<name>app-builder</name>
<description>Main application building orchestrator. Creates full-stack applications from natural language requests. Determines project type, selects tech stack, coordinates agents.</description>
<location>project</location>
</skill>

<skill>
<name>architecture</name>
<description>Architectural decision-making framework. Requirements analysis, trade-off evaluation, ADR documentation. Use when making architecture decisions or analyzing system design.</description>
<location>project</location>
</skill>

<skill>
<name>async-python-patterns</name>
<description>Master Python asyncio, concurrent programming, and async/await patterns for high-performance applications. Use when building async APIs, concurrent systems, or I/O-bound applications requiring non-blocking operations.</description>
<location>project</location>
</skill>

<skill>
<name>autonomous-agent-patterns</name>
<description>Design patterns for building autonomous coding agents. Covers tool integration, permission systems, browser automation, and human-in-the-loop workflows. Use when building AI agents, designing tool APIs, implementing permission systems, or creating autonomous coding assistants.</description>
<location>project</location>
</skill>

<skill>
<name>aws-serverless</name>
<description>Specialized skill for building production-ready serverless applications on AWS. Covers Lambda functions, API Gateway, DynamoDB, SQS/SNS event-driven patterns, SAM/CDK deployment, and cold start optimization.</description>
<location>project</location>
</skill>

<skill>
<name>backend-architect</name>
<description>Expert backend architect specializing in scalable API design,</description>
<location>project</location>
</skill>

<skill>
<name>backend-security-coder</name>
<description>Expert in secure backend coding practices specializing in input</description>
<location>project</location>
</skill>

<skill>
<name>bash-defensive-patterns</name>
<description>Master defensive Bash programming techniques for production-grade scripts. Use when writing robust shell scripts, CI/CD pipelines, or system utilities requiring fault tolerance and safety.</description>
<location>project</location>
</skill>

<skill>
<name>bash-linux</name>
<description>Bash/Linux terminal patterns. Critical commands, piping, error handling, scripting. Use when working on macOS or Linux systems.</description>
<location>project</location>
</skill>

<skill>
<name>behavioral-modes</name>
<description>AI operational modes (brainstorm, implement, debug, review, teach, ship, orchestrate). Use to adapt behavior based on task type.</description>
<location>project</location>
</skill>

<skill>
<name>brainstorming</name>
<description>Socratic questioning protocol + user communication. MANDATORY for complex requests, new features, or unclear requirements. Includes progress reporting and error handling.</description>
<location>project</location>
</skill>

<skill>
<name>browser-automation</name>
<description>Browser automation powers web testing, scraping, and AI agent interactions. The difference between a flaky script and a reliable system comes down to understanding selectors, waiting strategies, and anti-detection patterns. This skill covers Playwright (recommended) and Puppeteer, with patterns for testing, scraping, and agentic browser control. Key insight: Playwright won the framework war. Unless you need Puppeteer&apos;s stealth ecosystem or are Chrome-only, Playwright is the better choice in 202</description>
<location>project</location>
</skill>

<skill>
<name>canvas-design</name>
<description>Create beautiful visual art in .png and .pdf documents using design philosophy. You should use this skill when the user asks to create a poster, piece of art, design, or other static piece. Create original visual designs, never copying existing artists&apos; work to avoid copyright violations.</description>
<location>project</location>
</skill>

<skill>
<name>clean-code</name>
<description>Pragmatic coding standards - concise, direct, no over-engineering, no unnecessary comments</description>
<location>project</location>
</skill>

<skill>
<name>code-review-checklist</name>
<description>Code review guidelines covering code quality, security, and best practices.</description>
<location>project</location>
</skill>

<skill>
<name>database-design</name>
<description>Database design principles and decision-making. Schema design, indexing strategy, ORM selection, serverless databases.</description>
<location>project</location>
</skill>

<skill>
<name>deployment-procedures</name>
<description>Production deployment principles and decision-making. Safe deployment workflows, rollback strategies, and verification. Teaches thinking, not scripts.</description>
<location>project</location>
</skill>

<skill>
<name>doc-coauthoring</name>
<description>Guide users through a structured workflow for co-authoring documentation. Use when user wants to write documentation, proposals, technical specs, decision docs, or similar structured content. This workflow helps users efficiently transfer context, refine content through iteration, and verify the doc works for readers. Trigger when user mentions writing docs, creating proposals, drafting specs, or similar documentation tasks.</description>
<location>project</location>
</skill>

<skill>
<name>documentation-templates</name>
<description>Documentation templates and structure guidelines. README, API docs, code comments, and AI-friendly documentation.</description>
<location>project</location>
</skill>

<skill>
<name>game-development</name>
<description>Game development orchestrator. Routes to platform-specific skills based on project needs.</description>
<location>project</location>
</skill>

<skill>
<name>geo-fundamentals</name>
<description>Generative Engine Optimization for AI search engines (ChatGPT, Claude, Perplexity).</description>
<location>project</location>
</skill>

<skill>
<name>i18n-localization</name>
<description>Internationalization and localization patterns. Detecting hardcoded strings, managing translations, locale files, RTL support.</description>
<location>project</location>
</skill>

<skill>
<name>intelligent-routing</name>
<description>Automatic agent selection and intelligent task routing. Analyzes user requests and automatically selects the best specialist agent(s) without requiring explicit user mentions.</description>
<location>project</location>
</skill>

<skill>
<name>lint-and-validate</name>
<description>Automatic quality control, linting, and static analysis procedures. Use after every code modification to ensure syntax correctness and project standards. Triggers onKeywords: lint, format, check, validate, types, static analysis.</description>
<location>project</location>
</skill>

<skill>
<name>mcp-builder</name>
<description>MCP (Model Context Protocol) server building principles. Tool design, resource patterns, best practices.</description>
<location>project</location>
</skill>

<skill>
<name>mobile-design</name>
<description>Mobile-first design thinking and decision-making for iOS and Android apps. Touch interaction, performance patterns, platform conventions. Teaches principles, not fixed values. Use when building React Native, Flutter, or native mobile apps.</description>
<location>project</location>
</skill>

<skill>
<name>mobile-responsiveness</name>
<description>Build responsive, mobile-first web applications. Use when implementing responsive layouts, touch interactions, mobile navigation, or optimizing for various screen sizes. Triggers on responsive design, mobile-first, breakpoints, touch events, viewport.</description>
<location>project</location>
</skill>

<skill>
<name>nodejs-best-practices</name>
<description>Node.js development principles and decision-making. Framework selection, async patterns, security, and architecture. Teaches thinking, not copying.</description>
<location>project</location>
</skill>

<skill>
<name>parallel-agents</name>
<description>Multi-agent orchestration patterns. Use when multiple independent tasks can run with different domain expertise or when comprehensive analysis requires multiple perspectives.</description>
<location>project</location>
</skill>

<skill>
<name>performance-profiling</name>
<description>Performance profiling principles. Measurement, analysis, and optimization techniques.</description>
<location>project</location>
</skill>

<skill>
<name>plan-writing</name>
<description>Structured task planning with clear breakdowns, dependencies, and verification criteria. Use when implementing features, refactoring, or any multi-step work.</description>
<location>project</location>
</skill>

<skill>
<name>playwright-skill</name>
<description>Complete browser automation with Playwright. Auto-detects dev servers, writes clean test scripts to /tmp. Test pages, fill forms, take screenshots, check responsive design, validate UX, test login flows, check links, automate any browser task. Use when user wants to test websites, automate browser interactions, validate web functionality, or perform any browser-based testing.</description>
<location>project</location>
</skill>

<skill>
<name>powershell-windows</name>
<description>PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling.</description>
<location>project</location>
</skill>

<skill>
<name>python-patterns</name>
<description>Python development principles and decision-making. Framework selection, async patterns, type hints, project structure. Teaches thinking, not copying.</description>
<location>project</location>
</skill>

<skill>
<name>react-best-practices</name>
<description>React and Next.js performance optimization from Vercel Engineering. Use when building React components, optimizing performance, eliminating waterfalls, reducing bundle size, reviewing code for performance issues, or implementing server/client-side optimizations.</description>
<location>project</location>
</skill>

<skill>
<name>red-team-tactics</name>
<description>Red team tactics principles based on MITRE ATT&amp;CK. Attack phases, detection evasion, reporting.</description>
<location>project</location>
</skill>

<skill>
<name>seo-fundamentals</name>
<description>SEO fundamentals, E-E-A-T, Core Web Vitals, and Google algorithm principles.</description>
<location>project</location>
</skill>

<skill>
<name>server-management</name>
<description>Server management principles and decision-making. Process management, monitoring strategy, and scaling decisions. Teaches thinking, not commands.</description>
<location>project</location>
</skill>

<skill>
<name>systematic-debugging</name>
<description>4-phase systematic debugging methodology with root cause analysis and evidence-based verification. Use when debugging complex issues.</description>
<location>project</location>
</skill>

<skill>
<name>tailwind-patterns</name>
<description>Tailwind CSS v4 principles. CSS-first configuration, container queries, modern patterns, design token architecture.</description>
<location>project</location>
</skill>

<skill>
<name>tdd-workflow</name>
<description>Test-Driven Development workflow principles. RED-GREEN-REFACTOR cycle.</description>
<location>project</location>
</skill>

<skill>
<name>testing-patterns</name>
<description>Testing patterns and principles. Unit, integration, mocking strategies.</description>
<location>project</location>
</skill>

<skill>
<name>ux-design-systems</name>
<description>Build consistent design systems with tokens, components, and theming. Use when creating component libraries, implementing design tokens, building theme systems, or ensuring design consistency. Triggers on design system, design tokens, component library, theming, dark mode.</description>
<location>project</location>
</skill>

<skill>
<name>vulnerability-scanner</name>
<description>Advanced vulnerability analysis principles. OWASP 2025, Supply Chain Security, attack surface mapping, risk prioritization.</description>
<location>project</location>
</skill>

<skill>
<name>web-accessibility</name>
<description>Build accessible web applications following WCAG guidelines. Use when implementing ARIA patterns, keyboard navigation, screen reader support, or ensuring accessibility compliance. Triggers on accessibility, a11y, WCAG, ARIA, screen reader, keyboard navigation.</description>
<location>project</location>
</skill>

<skill>
<name>web-artifacts-builder</name>
<description>Suite of tools for creating elaborate, multi-component claude.ai HTML artifacts using modern frontend web technologies (React, Tailwind CSS, shadcn/ui). Use for complex artifacts requiring state management, routing, or shadcn/ui components - not for simple single-file HTML/JSX artifacts.</description>
<location>project</location>
</skill>

<skill>
<name>web-design-guidelines</name>
<description>Review UI code for Web Interface Guidelines compliance. Use when asked to &quot;review my UI&quot;, &quot;check accessibility&quot;, &quot;audit design&quot;, &quot;review UX&quot;, or &quot;check my site against best practices&quot;.</description>
<location>project</location>
</skill>

<skill>
<name>webapp-testing</name>
<description>Web application testing principles. E2E, Playwright, deep audit strategies.</description>
<location>project</location>
</skill>

</available_skills>
<!-- SKILLS_TABLE_END -->

</skills_system>
