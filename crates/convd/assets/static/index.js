/* ==========================================================================
   Convertor Dashboard - single-file module structure

   Rules applied in this refactor:
   - Keep everything in one file but separate logical modules with comments
   - Common utilities live in the "common" section
   - Removed dependency grid code (HTML doesn't contain it)
   - Use 4-space indent and double quotes throughout
   - Top-level functions use function declarations; lambdas use arrow syntax
   - Names slightly optimized for clarity
   ========================================================================== */

/* --------------------- common module --------------------- */
const queryParams = new URLSearchParams(location.search);
const endpointConfiguration = {
    ping: queryParams.get("ping") || "/",
    health: queryParams.get("health") || "/healthy",
    redis: queryParams.get("redis") || "/redis",
    info: queryParams.get("version") || "/version",
};

function selectOne(selector) {
    return document.querySelector(selector);
}

function tierForLatency(latencyMs, ok) {
    if (!ok) return "bad";
    if (latencyMs < 200) return "ok";
    if (latencyMs < 800) return "warn";
    return "bad";
}

function colorForTier(tier) {
    if (tier === "ok") return getComputedStyle(document.documentElement).getPropertyValue("--ok").trim();
    if (tier === "warn") return getComputedStyle(document.documentElement).getPropertyValue("--warn").trim();
    return getComputedStyle(document.documentElement).getPropertyValue("--bad").trim();
}

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

function renderRing(element, progressDegree, tier) {
    element.style.background = `conic-gradient(${colorForTier(tier)} ${progressDegree}deg, var(--ringTrack) ${progressDegree}deg 360deg)`;
}

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

/* initialize header and endpoint labels */
selectOne("#hostLabel").textContent = location.host || "—";
selectOne("#userAgentLabel").textContent = navigator.userAgent;
selectOne("#endpointLabel-ping").textContent = endpointConfiguration.ping;
selectOne("#endpointLabel-health").textContent = endpointConfiguration.health;
selectOne("#endpointLabel-redis").textContent = endpointConfiguration.redis;
selectOne("#endpointLabel-info").textContent = endpointConfiguration.info;

/* --------------------- history module --------------------- */
function historyStoreKey(name) {
    return `gdp_latency_history::${location.origin}::${name}`;
}

function loadLatencyHistory(name) {
    try {
        const raw = localStorage.getItem(historyStoreKey(name));
        if (!raw) return [];
        const arr = JSON.parse(raw);
        return Array.isArray(arr) ? arr : [];
    } catch (err) {
        return [];
    }
}

function saveLatencyHistory(name, list) {
    try {
        localStorage.setItem(historyStoreKey(name), JSON.stringify(list));
    } catch (err) {
        // ignore
    }
}

const historyName = "health";
let latencyHistory = loadLatencyHistory(historyName).slice(-500);

const sparkCanvas = selectOne("#latencySparkline");
const sparkCtx = sparkCanvas.getContext("2d");

function resizeSpark() {
    const dpr = Math.min(2, devicePixelRatio || 1);
    sparkCanvas.width = sparkCanvas.clientWidth * dpr;
    sparkCanvas.height = sparkCanvas.clientHeight * dpr;
    sparkCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
    drawSpark();
}

function percentile(values, q) {
    if (values.length === 0) return NaN;
    const copy = [ ...values ].sort((a, b) => a - b);
    const idx = Math.min(copy.length - 1, Math.floor((q / 100) * copy.length));
    return copy[idx];
}

function drawSpark() {
    const W = sparkCanvas.clientWidth, H = sparkCanvas.clientHeight;
    sparkCtx.clearRect(0, 0, W, H);
    if (latencyHistory.length < 2) {
        selectOne("#latencyStats").textContent = "—";
        return;
    }
    const sorted = [ ...latencyHistory ].sort((a, b) => a - b);
    const p50 = Math.round(percentile(sorted, 50));
    const p95 = Math.round(percentile(sorted, 95));
    const p99 = Math.round(percentile(sorted, 99));
    selectOne("#latencyStats").textContent = `p50 ${p50}ms · p95 ${p95}ms · p99 ${p99}ms · n=${latencyHistory.length}`;
    const max = Math.max(...latencyHistory);
    const stepX = W / (latencyHistory.length - 1);
    sparkCtx.strokeStyle = "#3ab4ff";
    sparkCtx.lineWidth = 2;
    sparkCtx.beginPath();
    latencyHistory.forEach((v, i) => {
        const x = i * stepX;
        const y = H - (v / max) * H;
        if (i === 0) sparkCtx.moveTo(x, y); else sparkCtx.lineTo(x, y);
    });
    sparkCtx.stroke();
}

addEventListener("resize", resizeSpark, { passive: true });
resizeSpark();

function pushHistoryAndRedraw(latencyMs) {
    if (!Number.isFinite(latencyMs)) return;
    latencyHistory.push(Math.max(1, Math.round(latencyMs)));
    if (latencyHistory.length > 500) latencyHistory.shift();
    saveLatencyHistory(historyName, latencyHistory);
    drawSpark();
}

/* --------------------- ping module --------------------- */
async function runPing() {
    const ring = selectOne(".card[data-kind=\"ping\"] .ring");
    renderRing(ring, 45, "warn");
    const res = await requestWithMetrics(endpointConfiguration.ping);
    const tier = tierForLatency(res.durationMs, res.ok);
    renderRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs) / 1000 * 270), tier);
    renderCard("ping", { ok: res.ok, status: res.status, latencyMs: res.durationMs });
    return res;
}

/* --------------------- health module --------------------- */
async function runHealth() {
    const ring = selectOne(".card[data-kind=\"health\"] .ring");
    renderRing(ring, 45, "warn");
    const res = await requestWithMetrics(endpointConfiguration.health);
    const tier = tierForLatency(res.durationMs, res.ok);
    renderRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs) / 1000 * 270), tier);
    renderCard("health", { ok: res.ok, status: res.status, latencyMs: res.durationMs });
    pushHistoryAndRedraw(res.durationMs);
    return res;
}

/* --------------------- redis module --------------------- */
async function runRedis() {
    const ring = selectOne(".card[data-kind=\"redis\"] .ring");
    renderRing(ring, 45, "warn");
    const res = await requestWithMetrics(endpointConfiguration.redis, { expectJson: true });
    let ok = res.ok;
    if (ok && res.data && Object.prototype.hasOwnProperty.call(res.data, "redis")) {
        ok = String(res.data.redis).toLowerCase() === "ok";
    }
    const tier = tierForLatency(res.durationMs, ok);
    renderRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs) / 1000 * 270), tier);
    renderCard("redis", { ok, status: res.status, latencyMs: res.durationMs });
    return res;
}

/* --------------------- info module --------------------- */
async function runInfo() {
    const ring = selectOne(".card[data-kind=\"info\"] .ring");
    renderRing(ring, 45, "warn");
    const res = await requestWithMetrics(endpointConfiguration.info, { expectJson: true });
    const ok = res.ok;
    const tier = tierForLatency(res.durationMs, ok);
    renderRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs) / 1000 * 270), tier);
    const d = res.data || {};
    const v = d.build?.version || d.git?.commit?.id || d.version || d.app?.version || "";
    const n = d.app?.name || d.name || "";
    const tm = d.build?.time || d.time || d.timestamp || "";
    const note = [ n, v && `@ ${v}`, tm && `· ${tm}` ].filter(Boolean).join(" ");
    renderCard("info", { ok, status: res.status, latencyMs: res.durationMs, note: note || "—" });
    return res;
}

/* --------------------- benchmark module --------------------- */
let benchmarkAbortController = null;
const benchEndpointInput = selectOne("#benchEndpointInput");
const benchTotalInput = selectOne("#benchTotalInput");
const benchConcurrencyInput = selectOne("#benchConcurrencyInput");
const benchTimeoutInput = selectOne("#benchTimeoutInput");
const benchStartButton = selectOne("#benchStartButton");
const benchAbortButton = selectOne("#benchAbortButton");
const benchProgressBar = selectOne("#benchProgressBar");
const benchStats = selectOne("#benchStats");

const benchCfgKey = `gdp_bench_cfg::${location.origin}`;
try {
    const prev = JSON.parse(localStorage.getItem(benchCfgKey) || "{}");
    if (prev.endpoint) benchEndpointInput.value = prev.endpoint;
    if (prev.total) benchTotalInput.value = prev.total;
    if (prev.concurrency) benchConcurrencyInput.value = prev.concurrency;
    if (prev.timeoutMs) benchTimeoutInput.value = prev.timeoutMs;
} catch (err) {
    // ignore
}

function saveBenchCfg() {
    try {
        localStorage.setItem(benchCfgKey, JSON.stringify({
            endpoint: benchEndpointInput.value,
            total: Number(benchTotalInput.value),
            concurrency: Number(benchConcurrencyInput.value),
            timeoutMs: Number(benchTimeoutInput.value),
        }));
    } catch (err) {
        // ignore
    }
}

function updateBenchmarkProgress(done, total) {
    const percent = total === 0 ? 0 : Math.round((done / total) * 100);
    benchProgressBar.style.width = percent + "%";
}

function pctl(values, q) {
    if (values.length === 0) return NaN;
    const a = [ ...values ].sort((x, y) => x - y);
    const idx = Math.min(a.length - 1, Math.floor((q / 100) * a.length));
    return a[idx];
}

async function runBenchmark() {
    const endpoint = benchEndpointInput.value || endpointConfiguration.health;
    const total = Number(benchTotalInput.value) || 100;
    const concurrency = Number(benchConcurrencyInput.value) || 8;
    const timeoutMs = Number(benchTimeoutInput.value) || 8000;
    saveBenchCfg();

    benchStartButton.disabled = true;
    benchAbortButton.disabled = false;
    updateBenchmarkProgress(0, total);
    benchStats.textContent = "running...";

    benchmarkAbortController = new AbortController();
    let completed = 0;
    let success = 0;
    let failure = 0;
    const latencies = [];

    const worker = async () => {
        while (true) {
            const index = completed;
            if (index >= total || benchmarkAbortController.signal.aborted) break;
            completed++;
            const res = await requestWithMetrics(endpoint, { timeoutMs });
            if (res.ok) {
                success++;
                latencies.push(res.durationMs);
            } else {
                failure++;
            }
            updateBenchmarkProgress(success + failure, total);
        }
    };
    await Promise.all(Array.from({ length: Math.min(concurrency, total) }, worker));

    const p50 = Math.round(pctl(latencies, 50));
    const p95 = Math.round(pctl(latencies, 95));
    const p99 = Math.round(pctl(latencies, 99));
    const min = latencies.length ? Math.round(Math.min(...latencies)) : NaN;
    const max = latencies.length ? Math.round(Math.max(...latencies)) : NaN;
    const errRate = total === 0 ? 0 : Math.round((failure / total) * 100);
    benchStats.textContent = `done ${success + failure}/${total} · ok ${success} · err ${failure} (${errRate}%)` + (latencies.length ? ` · min ${min}ms · p50 ${p50}ms · p95 ${p95}ms · p99 ${p99}ms · max ${max}ms` : "");

    benchStartButton.disabled = false;
    benchAbortButton.disabled = true;
    benchmarkAbortController = null;
}

function abortBenchmark() {
    if (benchmarkAbortController) {
        benchmarkAbortController.abort();
        benchStats.textContent += " · aborted";
        benchStartButton.disabled = false;
        benchAbortButton.disabled = true;
    }
}

selectOne("#benchStartButton").addEventListener("click", runBenchmark);
selectOne("#benchAbortButton").addEventListener("click", abortBenchmark);
selectOne("#runBenchmarkButton").addEventListener("click", () => {
    benchEndpointInput.value = endpointConfiguration.health;
    runBenchmark();
});

/* --------------------- wiring / small helpers --------------------- */
document.querySelectorAll("[data-run]").forEach((button) => {
    button.addEventListener("click", async () => {
        const kind = button.getAttribute("data-run");
        if (kind === "ping") await runPing();
        if (kind === "health") await runHealth();
        if (kind === "redis") await runRedis();
        if (kind === "info") await runInfo();
    });
});

selectOne("#runAllButton").addEventListener("click", async () => {
    await runPing();
    await runHealth();
    await runRedis();
    await runInfo();
});

selectOne("#latencyLabel-ping").textContent = "—";
selectOne("#latencyLabel-health").textContent = "—";
selectOne("#latencyLabel-redis").textContent = "—";
selectOne("#latencyLabel-info").textContent = "—";
