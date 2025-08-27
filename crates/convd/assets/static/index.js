/* ==========================================================================
    Convertor Dashboard — single-file modules

    Conventions:
    - Sections separated by short comment headers
    - 4-space indent, double quotes, semicolons
    - Modules initialized from main()
    ========================================================================== */

/* ----- Common ----- */
const queryParams = new URLSearchParams(location.search);

// endpoints come directly from query params or defaults; no central object needed

/**
 * Query a single DOM element using a selector.
 * @param {string} selector - CSS selector
 * @returns {Element|null}
 */
function selectOne(selector) {
    return document.querySelector(selector);
}

/**
 * Classify a latency value into a display tier.
 * @param {number} latencyMs
 * @param {boolean} ok
 * @returns {"ok"|"warn"|"bad"}
 */
function tierForLatency(latencyMs, ok) {
    if (!ok) return "bad";
    if (latencyMs < 200) return "ok";
    if (latencyMs < 800) return "warn";
    return "bad";
}

/**
 * Map a tier to a CSS color value (reads CSS variables).
 * @param {string} tier
 * @returns {string}
 */
function colorForTier(tier) {
    if (tier === "ok") return getComputedStyle(document.documentElement).getPropertyValue("--ok").trim();
    if (tier === "warn") return getComputedStyle(document.documentElement).getPropertyValue("--warn").trim();
    return getComputedStyle(document.documentElement).getPropertyValue("--bad").trim();
}

/**
 * Fetch wrapper that measures duration and optionally parses JSON.
 * Returns a result object and never throws (errors are returned as ok=false).
 * @param {string} url
 * @param {{timeoutMs?: number, expectJson?: boolean}} options
 * @returns {Promise<{ok:boolean,status:any,statusText:string,durationMs:number,data?:any,redirected:boolean}>}
 */
async function requestWithMetrics(url, options = {}) {
    const { timeoutMs = 8000, expectJson = false, ...init } = options;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort("timeout"), timeoutMs);
    const startedAt = performance.now();
    try {
        const response = await fetch(url, { cache: "no-store", mode: "same-origin", signal: controller.signal, ...init });
        const durationMs = performance.now() - startedAt;
        let parsed = undefined;
        if (expectJson) {
            try {
                parsed = await response.clone().json();
            } catch (err) {
                // ignore parse errors
            }
        }
        return { ok: response.ok, status: response.status, statusText: response.statusText, durationMs, data: parsed, redirected: response.redirected };
    } catch (error) {
        const durationMs = performance.now() - startedAt;
        return { ok: false, status: "ERR", statusText: String(error), durationMs, error, redirected: false };
    } finally {
        clearTimeout(timeoutId);
    }
}

/**
 * Render a conic-gradient ring on the provided element.
 * @param {Element} element
 * @param {number} progressDegree
 * @param {string} tier
 */
function renderRing(element, progressDegree, tier) {
    element.style.background = `conic-gradient(${colorForTier(tier)} ${progressDegree}deg, var(--ringTrack) ${progressDegree}deg 360deg)`;
}

/**
 * Update card DOM nodes (status dot, label, latency).
 * @param {string} kind
 * @param {{ok:boolean,status:any,latencyMs?:number,note?:string}} result
 */
function renderCard(kind, result) {
    const statusElement = selectOne(`#statusLabel-${kind}`);
    const dotElement = statusElement?.previousElementSibling;
    const latencyElement = selectOne(`#latencyLabel-${kind}`);
    const tier = tierForLatency(result.latencyMs ?? 0, result.ok);
    if (statusElement && dotElement && latencyElement) {
        statusElement.textContent = result.ok ? `OK (${result.status})` : `FAIL (${result.status})`;
        dotElement.className = `dot ${result.ok ? tier : "bad"}`;
        latencyElement.textContent = Number.isFinite(result.latencyMs) ? `${Math.round(result.latencyMs)}ms` : "—";
    }
    if (kind === "info" && result.note) {
        selectOne("#infoSummary").textContent = result.note;
    }
}

/* Endpoint labels are initialized in main() */

/* --------------------- history module --------------------- */

/* ----- Card: DashboardCard ----- */
/**
 * Reusable dashboard card that performs a single endpoint check and updates
 * its DOM nodes (ring, status, latency, optional info summary).
 */
class DashboardCard {
    constructor({ kind, endpoint }) {
        this.kind = kind;
        this.endpoint = endpoint;
        this.root = selectOne(`.card[data-kind="${kind}"]`);
        this.ring = this.root.querySelector(".ring");
        this.statusEl = selectOne(`#statusLabel-${kind}`);
        this.latencyEl = selectOne(`#latencyLabel-${kind}`);
        this.testButton = this.root.querySelector("[data-run]") || this.root.querySelector("button");

        // initialize UI labels
        const lbl = selectOne(`#endpointLabel-${this.kind}`);
        if (lbl) lbl.textContent = this.endpoint;

        // bind button
        if (this.testButton) this.testButton.addEventListener("click", () => this.run());
    }

    /**
     * Execute the check for this card's endpoint and update the UI.
     * @returns {Promise<{ok:boolean,status:any,durationMs:number,data?:any}>}
     */
    async run() {
        renderRing(this.ring, 45, "warn");
        const res = await requestWithMetrics(this.endpoint, { expectJson: this.kind === "redis" || this.kind === "info" });
        const ok = this._normalizeOk(res);
        const tier = tierForLatency(res.durationMs, ok);
        renderRing(this.ring, Math.min(330, 60 + Math.min(1000, res.durationMs) / 1000 * 270), tier);

        const note = this.kind === "info" ? this._extractInfoNote(res.data) : undefined;
        renderCard(this.kind, { ok, status: res.status, latencyMs: res.durationMs, note });
        return res;
    }

    /**
     * Normalize the "ok" result for special-case kinds (e.g. redis payloads).
     * @param {{ok:boolean,data?:any}} res
     * @returns {boolean}
     */
    _normalizeOk(res) {
        if (this.kind === "redis") {
            let ok = res.ok;
            if (ok && res.data && Object.prototype.hasOwnProperty.call(res.data, "redis")) {
                ok = String(res.data.redis).toLowerCase() === "ok";
            }
            return ok;
        }
        return res.ok;
    }

    /**
     * Extract a compact info summary string from an info endpoint payload.
     * @param {any} data
     * @returns {string}
     */
    _extractInfoNote(data) {
        const d = data || {};
        const v = d.build?.version || d.git?.commit?.id || d.version || d.app?.version || "";
        const n = d.app?.name || d.name || "";
        const tm = d.build?.time || d.time || d.timestamp || "";
        return [ n, v && `@ ${v}`, tm && `· ${tm}` ].filter(Boolean).join(" ") || "—";
    }

    /**
     * Update the endpoint for this card and refresh the endpoint label.
     * @param {string} path
     */
    setEndpoint(path) {
        this.endpoint = path;
        const lbl = selectOne(`#endpointLabel-${this.kind}`);
        if (lbl) lbl.textContent = path;
    }
}

/* Cards are instantiated in main() */

/* ----- History ----- */
/**
 * HistoryModule manages a rolling list of latencies and renders the
 * sparkline and percentile stats into the canvas and stat node.
 */
class HistoryModule {
    constructor() {
        this.historyName = "health";
        this.latencyHistory = this._loadLatencyHistory().slice(-500);
        this.sparkCanvas = selectOne("#latencySparkline");
        this.sparkCtx = this.sparkCanvas.getContext("2d");
        addEventListener("resize", () => this.resize(), { passive: true });
        this.resize();
    }

    /**
     * Storage key helper for persisting a named history.
     * @param {string} name
     * @returns {string}
     */
    _storeKey(name) {
        return `gdp_latency_history::${location.origin}::${name}`;
    }

    /**
     * Load persisted latency array from localStorage.
     * @returns {number[]}
     */
    _loadLatencyHistory() {
        try {
            const raw = localStorage.getItem(this._storeKey(this.historyName));
            if (!raw) return [];
            const arr = JSON.parse(raw);
            return Array.isArray(arr) ? arr : [];
        } catch (err) {
            return [];
        }
    }

    /**
     * Persist latency history array to localStorage.
     * @param {number[]} list
     */
    _saveLatencyHistory(list) {
        try {
            localStorage.setItem(this._storeKey(this.historyName), JSON.stringify(list));
        } catch (err) {
            // ignore
        }
    }

    /**
     * Resize the canvas for current DPR and redraw.
     */
    resize() {
        const dpr = Math.min(2, devicePixelRatio || 1);
        this.sparkCanvas.width = this.sparkCanvas.clientWidth * dpr;
        this.sparkCanvas.height = this.sparkCanvas.clientHeight * dpr;
        this.sparkCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
        this.draw();
    }

    /**
     * Compute a simple percentile from an array of numbers.
     * @param {number[]} values
     * @param {number} q percentile (0-100)
     * @returns {number}
     */
    percentile(values, q) {
        if (values.length === 0) return NaN;
        const copy = [ ...values ].sort((a, b) => a - b);
        const idx = Math.min(copy.length - 1, Math.floor((q / 100) * copy.length));
        return copy[idx];
    }

    /**
     * Draw the sparkline and update latency stats node.
     */
    draw() {
        const W = this.sparkCanvas.clientWidth, H = this.sparkCanvas.clientHeight;
        this.sparkCtx.clearRect(0, 0, W, H);
        if (this.latencyHistory.length < 2) {
            selectOne("#latencyStats").textContent = "—";
            return;
        }
        const sorted = [ ...this.latencyHistory ].sort((a, b) => a - b);
        const p50 = Math.round(this.percentile(sorted, 50));
        const p95 = Math.round(this.percentile(sorted, 95));
        const p99 = Math.round(this.percentile(sorted, 99));
        selectOne("#latencyStats").textContent = `p50 ${p50}ms · p95 ${p95}ms · p99 ${p99}ms · n=${this.latencyHistory.length}`;
        const max = Math.max(...this.latencyHistory);
        const stepX = W / (this.latencyHistory.length - 1);
        this.sparkCtx.strokeStyle = "#3ab4ff";
        this.sparkCtx.lineWidth = 2;
        this.sparkCtx.beginPath();
        this.latencyHistory.forEach((v, i) => {
            const x = i * stepX;
            const y = H - (v / max) * H;
            if (i === 0) this.sparkCtx.moveTo(x, y); else this.sparkCtx.lineTo(x, y);
        });
        this.sparkCtx.stroke();
    }

    /**
     * Push a latency sample into the history and redraw.
     * @param {number} latencyMs
     */
    push(latencyMs) {
        if (!Number.isFinite(latencyMs)) return;
        this.latencyHistory.push(Math.max(1, Math.round(latencyMs)));
        if (this.latencyHistory.length > 500) this.latencyHistory.shift();
        this._saveLatencyHistory(this.latencyHistory);
        this.draw();
    }
}

/* ----- Bench ----- */

class BenchModule {
    constructor(historyModule) {
        this.history = historyModule;
        this.benchEndpointInput = selectOne("#benchEndpointInput");
        this.benchTotalInput = selectOne("#benchTotalInput");
        this.benchConcurrencyInput = selectOne("#benchConcurrencyInput");
        this.benchTimeoutInput = selectOne("#benchTimeoutInput");
        this.benchStartButton = selectOne("#benchStartButton");
        this.benchAbortButton = selectOne("#benchAbortButton");
        this.benchProgressBar = selectOne("#benchProgressBar");
        this.benchStats = selectOne("#benchStats");

        this.benchCfgKey = `gdp_bench_cfg::${location.origin}`;
        try {
            const prev = JSON.parse(localStorage.getItem(this.benchCfgKey) || "{}");
            if (prev.endpoint) this.benchEndpointInput.value = prev.endpoint;
            if (prev.total) this.benchTotalInput.value = prev.total;
            if (prev.concurrency) this.benchConcurrencyInput.value = prev.concurrency;
            if (prev.timeoutMs) this.benchTimeoutInput.value = prev.timeoutMs;
        } catch (err) {
            // ignore
        }

        this.benchStartButton.addEventListener("click", () => this.run());
        this.benchAbortButton.addEventListener("click", () => this.abort());
        selectOne("#runBenchmarkButton").addEventListener("click", () => {
            this.benchEndpointInput.value = queryParams.get("health") || "/healthy";
            this.run();
        });
    }

    /**
     * Persist bench form values to localStorage.
     * @private
     */
    _saveCfg() {
        try {
            localStorage.setItem(this.benchCfgKey, JSON.stringify({
                endpoint: this.benchEndpointInput.value,
                total: Number(this.benchTotalInput.value),
                concurrency: Number(this.benchConcurrencyInput.value),
                timeoutMs: Number(this.benchTimeoutInput.value),
            }));
        } catch (err) {
            // ignore
        }
    }

    /**
     * Update visual progress bar width.
     * @param {number} done
     * @param {number} total
     * @private
     */
    _updateProgress(done, total) {
        const percent = total === 0 ? 0 : Math.round((done / total) * 100);
        this.benchProgressBar.style.width = percent + "%";
    }

    /**
     * Compute percentile used by the bench stats.
     * @param {number[]} values
     * @param {number} q
     * @returns {number}
     * @private
     */
    _pctl(values, q) {
        if (values.length === 0) return NaN;
        const a = [ ...values ].sort((x, y) => x - y);
        const idx = Math.min(a.length - 1, Math.floor((q / 100) * a.length));
        return a[idx];
    }

    /**
     * Run the benchmark using the form inputs and update history/stats.
     */
    async run() {
        const endpoint = this.benchEndpointInput.value || (queryParams.get("health") || "/healthy");
        const total = Number(this.benchTotalInput.value) || 100;
        const concurrency = Number(this.benchConcurrencyInput.value) || 8;
        const timeoutMs = Number(this.benchTimeoutInput.value) || 8000;
        this._saveCfg();

        this.benchStartButton.disabled = true;
        this.benchAbortButton.disabled = false;
        this._updateProgress(0, total);
        this.benchStats.textContent = "running...";

        this.benchmarkAbortController = new AbortController();
        let completed = 0;
        let success = 0;
        let failure = 0;
        const latencies = [];

        const worker = async () => {
            while (true) {
                const index = completed;
                if (index >= total || this.benchmarkAbortController.signal.aborted) break;
                completed++;
                const res = await requestWithMetrics(endpoint, { timeoutMs });
                if (res.ok) {
                    success++;
                    latencies.push(res.durationMs);
                    this.history.push(res.durationMs);
                } else {
                    failure++;
                }
                this._updateProgress(success + failure, total);
            }
        };
        await Promise.all(Array.from({ length: Math.min(concurrency, total) }, worker));

        const p50 = Math.round(this._pctl(latencies, 50));
        const p95 = Math.round(this._pctl(latencies, 95));
        const p99 = Math.round(this._pctl(latencies, 99));
        const min = latencies.length ? Math.round(Math.min(...latencies)) : NaN;
        const max = latencies.length ? Math.round(Math.max(...latencies)) : NaN;
        const errRate = total === 0 ? 0 : Math.round((failure / total) * 100);
        this.benchStats.textContent = `done ${success + failure}/${total} · ok ${success} · err ${failure} (${errRate}%)` + (latencies.length ? ` · min ${min}ms · p50 ${p50}ms · p95 ${p95}ms · p99 ${p99}ms · max ${max}ms` : "");

        this.benchStartButton.disabled = false;
        this.benchAbortButton.disabled = true;
        this.benchmarkAbortController = null;
    }

    /**
     * Abort an in-flight benchmark run.
     */
    abort() {
        if (this.benchmarkAbortController) {
            this.benchmarkAbortController.abort();
            this.benchStats.textContent += " · aborted";
            this.benchStartButton.disabled = false;
            this.benchAbortButton.disabled = true;
        }
    }
}

/* ----- Main / Wiring ----- */
/**
 * Entry point: wire up header, instantiate cards, history and bench modules,
 * and wire the run-all button.
 */
function main() {
    // initialize header
    selectOne("#hostLabel").textContent = location.host || "—";
    selectOne("#userAgentLabel").textContent = navigator.userAgent;
    // initialize endpoint labels
    selectOne("#endpointLabel-ping").textContent = queryParams.get("ping") || "/";
    selectOne("#endpointLabel-health").textContent = queryParams.get("health") || "/healthy";
    selectOne("#endpointLabel-redis").textContent = queryParams.get("redis") || "/redis";
    selectOne("#endpointLabel-info").textContent = queryParams.get("version") || "/version";

    // create dashboard cards (instantiation == initialization)
    const ping = new DashboardCard({ kind: "ping", endpoint: queryParams.get("ping") || "/" });
    const health = new DashboardCard({ kind: "health", endpoint: queryParams.get("health") || "/healthy" });
    const redis = new DashboardCard({ kind: "redis", endpoint: queryParams.get("redis") || "/redis" });
    const info = new DashboardCard({ kind: "info", endpoint: queryParams.get("version") || "/version" });

    // expose run all wiring
    selectOne("#runAllButton").addEventListener("click", async () => {
        await ping.run();
        await health.run();
        await redis.run();
        await info.run();
    });

    // initialize history and bench modules
    const history = new HistoryModule();
    const bench = new BenchModule(history);

    // initialize latency labels
    selectOne("#latencyLabel-ping").textContent = "—";
    selectOne("#latencyLabel-health").textContent = "—";
    selectOne("#latencyLabel-redis").textContent = "—";
    selectOne("#latencyLabel-info").textContent = "—";
}

main();
