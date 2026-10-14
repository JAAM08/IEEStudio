// Graba la app siguiendo el guion y anota cuándo empieza cada parte.
const { chromium } = require('playwright');
const fs = require('fs');

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const plan = JSON.parse(fs.readFileSync(__dirname + '/plan.json', 'utf8'));

const CURSOR = () => {
  window.addEventListener('DOMContentLoaded', () => {
    const c = document.createElement('div');
    c.id = 'cursor-falso';
    c.innerHTML = '<svg width="26" height="26" viewBox="0 0 26 26"><path d="M3 2 L3 20 L8 15.5 L11.5 23 L14.5 21.6 L11 14.2 L17.5 14.2 Z" fill="#fff" stroke="#111" stroke-width="1.6" stroke-linejoin="round"/></svg>';
    Object.assign(c.style, { position: 'fixed', left: '0', top: '0', zIndex: 99999, pointerEvents: 'none', transform: 'translate(-100px,-100px)', filter: 'drop-shadow(0 2px 3px rgba(0,0,0,.5))' });
    document.body.appendChild(c);
    const st = document.createElement('style');
    st.textContent = '.onda{position:fixed;z-index:99998;pointer-events:none;width:44px;height:44px;margin:-22px 0 0 -22px;border-radius:50%;border:3px solid #f5c518;animation:onda .6s ease-out forwards}@keyframes onda{from{transform:scale(.3);opacity:1}to{transform:scale(1.4);opacity:0}}';
    document.head.appendChild(st);
    window.addEventListener('pointermove', (e) => { c.style.transform = `translate(${e.clientX - 3}px, ${e.clientY - 2}px)`; }, true);
    window.addEventListener('pointerdown', (e) => {
      const o = document.createElement('div'); o.className = 'onda';
      o.style.left = e.clientX + 'px'; o.style.top = e.clientY + 'px';
      document.body.appendChild(o); setTimeout(() => o.remove(), 700);
    }, true);
  });
};

(async () => {
  const b = await chromium.launch({ args: ['--force-device-scale-factor=1.3333333'] });
  const ctx = await b.newContext({
    viewport: { width: 1440, height: 810 }, deviceScaleFactor: 1.3333333,
    recordVideo: { dir: __dirname + '/grabacion', size: { width: 1920, height: 1080 } },
  });
  const p = await ctx.newPage();
  const t0 = Date.now();
  const now = () => (Date.now() - t0) / 1000;
  await p.addInitScript(CURSOR);
  await p.goto('http://localhost:8765/app/');
  await p.waitForFunction(() => document.getElementById('carga').hidden, null, { timeout: 120000 });
  await sleep(1500);
  let mx = 720, my = 470;
  await p.mouse.move(mx, my);
  const moveTo = async (x, y) => {
    const d = Math.hypot(x - mx, y - my);
    await p.mouse.move(x, y, { steps: Math.max(8, Math.round(d / 18)) });
    mx = x; my = y;
  };
  const log = { ready: now(), segments: [] };
  for (const seg of plan) {
    const start = now();
    for (const a of seg.actions) {
      if (a[0] === 'move') { await moveTo(a[1], a[2]); }
      else if (a[0] === 'click') { await moveTo(a[1], a[2]); await sleep(250); await p.mouse.click(a[1], a[2]); await sleep(300); }
      else if (a[0] === 'key') { await p.keyboard.press(a[1]); }
      else if (a[0] === 'wait') { await sleep(a[1]); }
      else if (a[0] === 'scroll') { await moveTo(a[1], a[2]); for (let k = 0; k < 30; k++) { await p.mouse.wheel(0, a[3] / 30); await sleep(60); } }
    }
    // Lo que tarde la app en responder se recorta al montar; la voz empieza
    // cuando la pantalla ya cambió.
    const ready = now();
    await sleep(seg.length * 1000);
    log.segments.push({ id: seg.id, start, ready, end: now() });
  }
  log.end = now();
  fs.writeFileSync(__dirname + '/tiempos.json', JSON.stringify(log, null, 1));
  await ctx.close();
  await b.close();
})();
