// Dibuja el personaje del video en sus variantes (boca y ojos) como PNG.
const { chromium } = require('playwright');

const skin = '#c98d64', skinShade = '#b57a53', dark = '#1b1e23', hat = '#f5c518', hatShade = '#dcae0e';

function svg(mouth, eyes) {
  const eyesSvg = eyes === 'open'
    ? `<ellipse cx="128" cy="157" rx="6.5" ry="8.5" fill="${dark}"/><ellipse cx="172" cy="157" rx="6.5" ry="8.5" fill="${dark}"/>
       <circle cx="130" cy="154" r="2" fill="#fff"/><circle cx="174" cy="154" r="2" fill="#fff"/>`
    : `<path d="M121 158 q7 4 14 0 M165 158 q7 4 14 0" stroke="${dark}" stroke-width="3.5" fill="none" stroke-linecap="round"/>`;
  const mouths = {
    closed: `<path d="M136 191 q14 10 28 0" stroke="#5a2420" stroke-width="4" fill="none" stroke-linecap="round"/>`,
    half: `<path d="M135 189 q15 2 30 0 q-2 11 -15 11 q-13 0 -15 -11z" fill="#5a2420"/><path d="M140 195 q10 5 20 0" stroke="#d9686a" stroke-width="3" fill="none" stroke-linecap="round"/>`,
    open: `<path d="M134 186 q16 -2 32 0 q-1 19 -16 19 q-15 0 -16 -19z" fill="#4a1d1a"/><path d="M139 199 q11 7 22 0 q-3 6 -11 6 q-8 0 -11 -6z" fill="#d9686a"/><path d="M137 187 h26 v4 h-26z" fill="#fff" opacity=".9"/>`,
  };
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 300" width="300" height="300">
  <defs><clipPath id="c"><circle cx="150" cy="150" r="143"/></clipPath>
    <radialGradient id="bg" cx="50%" cy="35%" r="70%"><stop offset="0" stop-color="#2b3038"/><stop offset="1" stop-color="#16181c"/></radialGradient></defs>
  <circle cx="150" cy="150" r="146" fill="url(#bg)"/>
  <g clip-path="url(#c)">
    <path d="M30 310 Q38 228 150 222 Q262 228 270 310 Z" fill="#2c5a87"/>
    <path d="M118 224 L150 262 L182 224 Z" fill="#e9ecef"/>
    <path d="M130 223 L150 250 L170 223 Z" fill="${skin}"/>
    <path d="M104 236 l-8 60 M196 236 l8 60" stroke="#244b72" stroke-width="5"/>
    <rect x="200" y="252" width="30" height="20" rx="3" fill="#244b72"/>
    <path d="M206 258 h18 M206 264 h12" stroke="${hat}" stroke-width="2.5"/>
    <rect x="134" y="196" width="32" height="34" rx="10" fill="${skinShade}"/>
    <ellipse cx="93" cy="160" rx="10" ry="14" fill="${skinShade}"/>
    <ellipse cx="207" cy="160" rx="10" ry="14" fill="${skinShade}"/>
    <ellipse cx="150" cy="158" rx="58" ry="66" fill="${skin}"/>
    <path d="M110 140 q10 -7 22 -3 M168 137 q12 -4 22 3" stroke="#3a2a22" stroke-width="5" fill="none" stroke-linecap="round"/>
    ${eyesSvg}
    <path d="M150 162 q-6 14 0 18 q4 1 7 -1" stroke="${skinShade}" stroke-width="3.5" fill="none" stroke-linecap="round"/>
    <ellipse cx="116" cy="182" rx="10" ry="6" fill="#d9776a" opacity=".35"/><ellipse cx="184" cy="182" rx="10" ry="6" fill="#d9776a" opacity=".35"/>
    ${mouths[mouth]}
    <path d="M86 128 Q86 60 150 56 Q214 60 214 128 Z" fill="${hat}"/>
    <path d="M150 56 Q140 90 142 128 L158 128 Q160 90 150 56Z" fill="${hatShade}"/>
    <rect x="72" y="120" width="156" height="17" rx="8.5" fill="${hatShade}"/>
    <rect x="72" y="120" width="156" height="7" rx="3.5" fill="${hat}"/>
    <path d="M118 78 l-11 17 h7 l-3 13 l12 -18 h-7z" fill="${dark}"/>
  </g>
  <circle cx="150" cy="150" r="143" fill="none" stroke="${hat}" stroke-width="7"/>
</svg>`;
}

(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 300, height: 300 } });
  for (const mouth of ['closed', 'half', 'open']) for (const eyes of ['open', 'shut']) {
    await p.setContent(`<html><body style="margin:0;background:transparent">${svg(mouth, eyes)}</body></html>`);
    await p.screenshot({ path: `${__dirname}/p-${mouth}-${eyes}.png`, omitBackground: true });
  }
  await b.close();
})();
