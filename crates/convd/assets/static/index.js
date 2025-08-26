const queryParams = new URLSearchParams(location.search);
const endpointConfiguration = {
  ping  : queryParams.get('ping')   || '/',
  health: queryParams.get('health') || '/healthy',
  redis : queryParams.get('redis')  || '/redis',
  info  : queryParams.get('version')   || '/version',
  deps  : queryParams.get('deps')   || '',
  assets: queryParams.get('assets') || ''
};
const selectOne = (sel) => document.querySelector(sel);
const classifyTier = (latencyMs, ok) => (!ok) ? 'bad' : latencyMs < 200 ? 'ok' : latencyMs < 800 ? 'warn' : 'bad';
const colorByTier = (tier) =>
  tier === 'ok' ? getComputedStyle(document.documentElement).getPropertyValue('--ok').trim()
    : tier === 'warn' ? getComputedStyle(document.documentElement).getPropertyValue('--warn').trim()
    : getComputedStyle(document.documentElement).getPropertyValue('--bad').trim();

async function httpRequestWithMetrics(url, options = {}) {
  const { timeoutMs = 8000, expectJson = false, ...init } = options;
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort('timeout'), timeoutMs);
  const startedAt = performance.now();
  try {
    const response = await fetch(url, { cache:'no-store', mode:'same-origin', signal:controller.signal, ...init });
    const durationMs = performance.now() - startedAt;
    let parsed = undefined;
    if (expectJson) { try { parsed = await response.clone().json(); } catch {} }
    return { ok:response.ok, status:response.status, statusText:response.statusText, durationMs, data:parsed, redirected:response.redirected };
  } catch (error) {
    const durationMs = performance.now() - startedAt;
    return { ok:false, status:'ERR', statusText:String(error), durationMs, error, redirected:false };
  } finally { clearTimeout(timeoutId); }
}

function updateRing(element, progressDegree, tier) {
  element.style.background = `conic-gradient(${colorByTier(tier)} ${progressDegree}deg, var(--ringTrack) ${progressDegree}deg 360deg)`;
}
function updateCard(kind, result) {
  const statusElement = selectOne(`#statusLabel-${kind}`);
  const dotElement = statusElement?.previousElementSibling;
  const latencyElement = selectOne(`#latencyLabel-${kind}`);
  const tier = classifyTier(result.latencyMs ?? 0, result.ok);
  if (statusElement && dotElement && latencyElement) {
    statusElement.textContent = result.ok ? `OK (${result.status})` : `FAIL (${result.status})`;
    dotElement.className = `dot ${result.ok ? tier : 'bad'}`;
    latencyElement.textContent = Number.isFinite(result.latencyMs) ? `${Math.round(result.latencyMs)}ms` : '—';
  }
  if (kind === 'info' && result.note) {
    selectOne('#infoSummary').textContent = result.note;
  }
}

selectOne('#hostLabel').textContent = location.host || '—';
selectOne('#userAgentLabel').textContent = navigator.userAgent;
selectOne('#endpointLabel-ping').textContent   = endpointConfiguration.ping;
selectOne('#endpointLabel-health').textContent = endpointConfiguration.health;
selectOne('#endpointLabel-redis').textContent  = endpointConfiguration.redis;
selectOne('#endpointLabel-info').textContent   = endpointConfiguration.info;

const historyStoreKey = (name) => `gdp_latency_history::${location.origin}::${name}`;
function loadLatencyHistory(name) {
  try {
    const raw = localStorage.getItem(historyStoreKey(name));
    if (!raw) return [];
    const arr = JSON.parse(raw);
    return Array.isArray(arr) ? arr : [];
  } catch { return []; }
}
function saveLatencyHistory(name, list) {
  try { localStorage.setItem(historyStoreKey(name), JSON.stringify(list)); } catch {}
}
const historyName = 'health';
let latencyHistory = loadLatencyHistory(historyName).slice(-500);

const sparkCanvas = selectOne('#latencySparkline');
const sparkCtx = sparkCanvas.getContext('2d');
function resizeSpark() {
  const dpr = Math.min(2, devicePixelRatio || 1);
  sparkCanvas.width  = sparkCanvas.clientWidth * dpr;
  sparkCanvas.height = sparkCanvas.clientHeight * dpr;
  sparkCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
  drawSpark();
}
function percentile(values, q) {
  if (values.length === 0) return NaN;
  const copy = [...values].sort((a,b)=>a-b);
  const idx = Math.min(copy.length-1, Math.floor(q/100 * copy.length));
  return copy[idx];
}
function drawSpark() {
  const W = sparkCanvas.clientWidth, H = sparkCanvas.clientHeight;
  sparkCtx.clearRect(0,0,W,H);
  if (latencyHistory.length < 2) { selectOne('#latencyStats').textContent = '—'; return; }
  const sorted = [...latencyHistory].sort((a,b)=>a-b);
  const p50=Math.round(percentile(sorted,50)), p95=Math.round(percentile(sorted,95)), p99=Math.round(percentile(sorted,99));
  selectOne('#latencyStats').textContent = `p50 ${p50}ms · p95 ${p95}ms · p99 ${p99}ms · n=${latencyHistory.length}`;
  const max = Math.max(...latencyHistory);
  const stepX = W / (latencyHistory.length-1);
  sparkCtx.strokeStyle = '#3ab4ff'; sparkCtx.lineWidth = 2; sparkCtx.beginPath();
  latencyHistory.forEach((v,i)=>{
    const x = i*stepX, y = H - (v/max)*H;
    if(i===0) sparkCtx.moveTo(x,y); else sparkCtx.lineTo(x,y);
  });
  sparkCtx.stroke();
}
addEventListener('resize', resizeSpark, { passive:true });
resizeSpark();

function pushHistoryAndRedraw(latencyMs) {
  if (!Number.isFinite(latencyMs)) return;
  latencyHistory.push(Math.max(1, Math.round(latencyMs)));
  if (latencyHistory.length > 500) latencyHistory.shift();
  saveLatencyHistory(historyName, latencyHistory);
  drawSpark();
}

async function runPing() {
  const ring = selectOne('.card[data-kind="ping"] .ring');
  updateRing(ring, 45, 'warn');
  const res = await httpRequestWithMetrics(endpointConfiguration.ping);
  const tier = classifyTier(res.durationMs, res.ok);
  updateRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs)/1000*270), tier);
  updateCard('ping', { ok:res.ok, status:res.status, latencyMs:res.durationMs });
  return res;
}
async function runHealth() {
  const ring = selectOne('.card[data-kind="health"] .ring');
  updateRing(ring, 45, 'warn');
  const res = await httpRequestWithMetrics(endpointConfiguration.health);
  const tier = classifyTier(res.durationMs, res.ok);
  updateRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs)/1000*270), tier);
  updateCard('health', { ok:res.ok, status:res.status, latencyMs:res.durationMs });
  pushHistoryAndRedraw(res.durationMs);
  return res;
}
async function runRedis() {
  const ring = selectOne('.card[data-kind="redis"] .ring');
  updateRing(ring, 45, 'warn');
  const res = await httpRequestWithMetrics(endpointConfiguration.redis, { expectJson:true });
  let ok = res.ok;
  if (ok && res.data && Object.prototype.hasOwnProperty.call(res.data, 'redis')) {
    ok = String(res.data.redis).toLowerCase() === 'ok';
  }
  const tier = classifyTier(res.durationMs, ok);
  updateRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs)/1000*270), tier);
  updateCard('redis', { ok, status:res.status, latencyMs:res.durationMs });
  return res;
}
async function runInfo() {
  const ring = selectOne('.card[data-kind="info"] .ring');
  updateRing(ring, 45, 'warn');
  const res = await httpRequestWithMetrics(endpointConfiguration.info, { expectJson:true });
  const ok = res.ok;
  const tier = classifyTier(res.durationMs, ok);
  updateRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs)/1000*270), tier);
  const d = res.data || {};
  const v = d.build?.version || d.git?.commit?.id || d.version || d.app?.version || '';
  const n = d.app?.name || d.name || '';
  const tm = d.build?.time || d.time || d.timestamp || '';
  const note = [n, v && `@ ${v}`, tm && `· ${tm}`].filter(Boolean).join(' ');
  updateCard('info', { ok, status:res.status, latencyMs:res.durationMs, note: note || '—' });
  return res;
}

function parseDeps(input) {
  const list = [];
  if (!input) return list;
  input.split(',').forEach(pair=>{
    const idx = pair.indexOf(':');
    if (idx>0) {
      const name = decodeURIComponent(pair.slice(0, idx).trim());
      const path = pair.slice(idx+1).trim();
      if (name && path) list.push({name, path});
    }
  });
  return list;
}


async function runDependency(i) {
  const d = depsList[i]; if (!d) return;
  const ring = depsGrid.children[i].querySelector('.ring');
  updateRing(ring, 45, 'warn');
  const res = await httpRequestWithMetrics(d.path);
  const tier = classifyTier(res.durationMs, res.ok);
  updateRing(ring, Math.min(330, 60 + Math.min(1000, res.durationMs)/1000*270), tier);
  const statusEl = selectOne(`#depStatus-${i}`), dot = statusEl.previousElementSibling, msEl = selectOne(`#depLatency-${i}`);
  statusEl.textContent = res.ok ? `OK (${res.status})` : `FAIL (${res.status})`;
  dot.className = `dot ${res.ok ? tier : 'bad'}`;
  msEl.textContent = Number.isFinite(res.durationMs) ? `${Math.round(res.durationMs)}ms` : '—';
}
selectOne('#depsRunAllButton').addEventListener('click', async ()=>{
  for (let i=0;i<depsList.length;i++) await runDependency(i);
});

function parseAssets(input) {
  if (!input) return [];
  return input.split(',').map(s=>s.trim()).filter(Boolean);
}
const assetsInput = selectOne('#assetsInput');
const assetsConcurrencyInput = selectOne('#assetsConcurrencyInput');
const assetsProgressBar = selectOne('#assetsProgressBar');
const assetsStats = selectOne('#assetsStats');

function setAssetsProgress(done, total) {
  const percent = total===0 ? 0 : Math.round((done/total)*100);
  assetsProgressBar.style.width = percent + '%';
}

selectOne('#assetsStartButton').addEventListener('click', async ()=>{
  const list = parseAssets(assetsInput.value || endpointConfiguration.assets);
  const concurrency = Number(assetsConcurrencyInput.value) || 4;
  if (list.length === 0) { assetsStats.textContent = '—'; setAssetsProgress(0,1); return; }

  let done=0, ok=0, fail=0;
  setAssetsProgress(0, list.length);
  assetsStats.textContent = 'running...';

  const runOne = async (url) => {
    const res = await httpRequestWithMetrics(url, { timeoutMs: 12000 });
    done++; if (res.ok) ok++; else fail++;
    setAssetsProgress(done, list.length);
  };
  const q = [...list];
  const workers = Array.from({length: Math.min(concurrency, q.length)}, async ()=>{
    while(q.length){
      const url = q.shift();
      try{ await runOne(url);}catch{}
    }
  });
  await Promise.all(workers);
  assetsStats.textContent = `done ${done} · ok ${ok} · err ${fail}`;
});

let benchmarkAbortController = null;
const benchEndpointInput = selectOne('#benchEndpointInput');
const benchTotalInput = selectOne('#benchTotalInput');
const benchConcurrencyInput = selectOne('#benchConcurrencyInput');
const benchTimeoutInput = selectOne('#benchTimeoutInput');
const benchStartButton = selectOne('#benchStartButton');
const benchAbortButton = selectOne('#benchAbortButton');
const benchProgressBar = selectOne('#benchProgressBar');
const benchStats = selectOne('#benchStats');

const benchCfgKey = `gdp_bench_cfg::${location.origin}`;
try {
  const prev = JSON.parse(localStorage.getItem(benchCfgKey) || '{}');
  if (prev.endpoint) benchEndpointInput.value = prev.endpoint;
  if (prev.total) benchTotalInput.value = prev.total;
  if (prev.concurrency) benchConcurrencyInput.value = prev.concurrency;
  if (prev.timeoutMs) benchTimeoutInput.value = prev.timeoutMs;
} catch {}

function saveBenchCfg() {
  try {
    localStorage.setItem(benchCfgKey, JSON.stringify({
      endpoint: benchEndpointInput.value,
      total: Number(benchTotalInput.value),
      concurrency: Number(benchConcurrencyInput.value),
      timeoutMs: Number(benchTimeoutInput.value)
    }));
  } catch {}
}

function updateBenchmarkProgress(done, total) {
  const percent = total === 0 ? 0 : Math.round((done/total)*100);
  benchProgressBar.style.width = percent + '%';
}
function pctl(values, q) {
  if (values.length === 0) return NaN;
  const a = [...values].sort((x,y)=>x-y);
  const idx = Math.min(a.length-1, Math.floor(q/100*a.length));
  return a[idx];
}
async function runBenchmark() {
  const endpoint = benchEndpointInput.value || endpointConfiguration.health;
  const total = Number(benchTotalInput.value) || 100;
  const concurrency = Number(benchConcurrencyInput.value) || 8;
  const timeoutMs = Number(benchTimeoutInput.value) || 8000;
  saveBenchCfg();

  benchStartButton.disabled = true; benchAbortButton.disabled = false;
  updateBenchmarkProgress(0, total);
  benchStats.textContent = 'running...';

  benchmarkAbortController = new AbortController();
  let completed = 0, success = 0, failure = 0;
  const latencies = [];

  const worker = async () => {
    while (true) {
      const index = completed;
      if (index >= total || benchmarkAbortController.signal.aborted) break;
      completed++;
      const res = await httpRequestWithMetrics(endpoint, { timeoutMs });
      if (res.ok) { success++; latencies.push(res.durationMs); } else { failure++; }
      updateBenchmarkProgress(success + failure, total);
    }
  };
  await Promise.all(Array.from({length: Math.min(concurrency, total)}, worker));

  const p50 = Math.round(pctl(latencies, 50));
  const p95 = Math.round(pctl(latencies, 95));
  const p99 = Math.round(pctl(latencies, 99));
  const min = Math.round(Math.min(...latencies));
  const max = Math.round(Math.max(...latencies));
  const errRate = total === 0 ? 0 : Math.round((failure/total)*100);
  benchStats.textContent =
    `done ${success+failure}/${total} · ok ${success} · err ${failure} (${errRate}%)` +
    (latencies.length ? ` · min ${min}ms · p50 ${p50}ms · p95 ${p95}ms · p99 ${p99}ms · max ${max}ms` : '');

  benchStartButton.disabled = false; benchAbortButton.disabled = true;
  benchmarkAbortController = null;
}
function abortBenchmark() {
  if (benchmarkAbortController) {
    benchmarkAbortController.abort();
    benchStats.textContent += ' · aborted';
    benchStartButton.disabled = false; benchAbortButton.disabled = true;
  }
}
selectOne('#benchStartButton').addEventListener('click', runBenchmark);
selectOne('#benchAbortButton').addEventListener('click', abortBenchmark);
selectOne('#runBenchmarkButton').addEventListener('click', ()=>{ benchEndpointInput.value = endpointConfiguration.health; runBenchmark(); });

document.querySelectorAll('[data-run]').forEach(button=>{
  button.addEventListener('click', async ()=>{
    const kind = button.getAttribute('data-run');
    if (kind === 'ping')   await runPing();
    if (kind === 'health') await runHealth();
    if (kind === 'redis')  await runRedis();
    if (kind === 'info')   await runInfo();
  });
});
selectOne('#runAllButton').addEventListener('click', async ()=>{
  await runPing(); await runHealth(); await runRedis(); await runInfo();
  if (depsList.length) { for (let i=0;i<depsList.length;i++) await runDependency(i); }
});

const depsList = (function parseDeps(input) {
  const list = [];
  if (!input) return list;
  input.split(',').forEach(pair=>{
    const idx = pair.indexOf(':');
    if (idx>0) {
      const name = decodeURIComponent(pair.slice(0, idx).trim());
      const path = pair.slice(idx+1).trim();
      if (name && path) list.push({name, path});
    }
  });
  return list;
})(endpointConfiguration.deps);

const depsGrid = selectOne('#depsGrid');
(function renderDeps() {
  depsGrid.innerHTML = '';
  depsList.forEach((d, i)=>{
    const item = document.createElement('div');
    item.className = 'dep-item';
    item.innerHTML = `
      <div class="ring"><span id="depLatency-${i}">—</span></div>
      <div class="kv">
        <div><b>${d.name}</b> <span class="badge"><span class="dot idle"></span><span id="depStatus-${i}">IDLE</span></span></div>
        <div class="muted">GET <code>${d.path}</code></div>
        <div class="row"><button data-dep-index="${i}">Test</button></div>
      </div>`;
    depsGrid.appendChild(item);
  });
  depsGrid.querySelectorAll('button[data-dep-index]').forEach(btn=>{
    btn.addEventListener('click', async ()=>{
      const i = Number(btn.getAttribute('data-dep-index'));
      await runDependency(i);
    });
  });
})();

selectOne('#latencyLabel-ping').textContent   = '—';
selectOne('#latencyLabel-health').textContent = '—';
selectOne('#latencyLabel-redis').textContent  = '—';
selectOne('#latencyLabel-info').textContent   = '—';
