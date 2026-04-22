const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const API = 'https://api.library.loxone.com';
const OUT_DIR = path.join('/home/amy/src/lox/docs/library');

async function main() {
  fs.mkdirSync(OUT_DIR, { recursive: true });
  fs.mkdirSync(path.join(OUT_DIR, 'plugins'), { recursive: true });

  // Use Windows Chrome via channel
  const browser = await chromium.launch({ 
    headless: false,
    executablePath: '/mnt/c/Program Files/Google/Chrome/Application/chrome.exe',
  });
  const context = await browser.newContext();
  const page = await context.newPage();

  console.log('Opening library login page...');
  await page.goto('https://library.loxone.com/login');
  
  console.log('>>> Please log in, then press Enter here <<<');
  await new Promise(resolve => process.stdin.once('data', resolve));

  // Get cookies
  const cookies = await context.cookies();
  const sess = cookies.find(c => c.name === 'PHP_SESSID_LOXONE_LIBRARY');
  const cookieHeader = sess ? `PHP_SESSID_LOXONE_LIBRARY=${sess.value}` : '';
  console.log(sess ? 'Session cookie acquired.' : 'No session cookie — trying without auth...');

  async function apiPost(endpoint, body = {}) {
    const resp = await fetch(`${API}${endpoint}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Origin': 'https://library.loxone.com', 'Cookie': cookieHeader },
      body: JSON.stringify(body),
    });
    return resp.json();
  }

  console.log('\nFetching metadata...');
  const categories = await apiPost('/loadCategories');
  fs.writeFileSync(path.join(OUT_DIR, 'categories.json'), JSON.stringify(categories, null, 2));
  const brands = await apiPost('/loadBrands');
  fs.writeFileSync(path.join(OUT_DIR, 'brands.json'), JSON.stringify(brands, null, 2));
  const technologies = await apiPost('/loadTechnologies');
  fs.writeFileSync(path.join(OUT_DIR, 'technologies.json'), JSON.stringify(technologies, null, 2));
  const filters = await apiPost('/loadFilters');
  fs.writeFileSync(path.join(OUT_DIR, 'filters.json'), JSON.stringify(filters, null, 2));
  console.log(`  ${Object.keys(categories).length} categories, ${Object.keys(brands).length} brands`);

  console.log('\nFetching plugin list...');
  const plugins = await apiPost('/loadFilteredPlugins', { language: 'en' });
  const list = Object.values(plugins);
  console.log(`  ${list.length} plugins`);
  fs.writeFileSync(path.join(OUT_DIR, 'plugins.json'), JSON.stringify(plugins, null, 2));

  console.log('\nDownloading configs...');
  let ok = 0, fail = 0;
  for (const p of list) {
    const dir = path.join(OUT_DIR, 'plugins', p.id);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.join(dir, 'meta.json'), JSON.stringify(p, null, 2));
    try {
      const resp = await fetch(`${API}/downloader/config/${p.id}`, {
        headers: { 'Origin': 'https://library.loxone.com', 'Cookie': cookieHeader },
      });
      if (resp.ok) {
        const buf = Buffer.from(await resp.arrayBuffer());
        fs.writeFileSync(path.join(dir, 'config.zip'), buf);
        // Extract ZIP entries
        const AdmZip = require('adm-zip');
        try {
          const zip = new AdmZip(buf);
          for (const e of zip.getEntries()) fs.writeFileSync(path.join(dir, e.entryName), e.getData());
        } catch(_) {}
        ok++;
        process.stdout.write(`  ${ok}/${list.length} ${p.name.substring(0,50).padEnd(50)}\r`);
      } else { fail++; console.log(`  ✗ ${p.name}: ${resp.status}`); }
    } catch(e) { fail++; console.log(`  ✗ ${p.name}: ${e.message}`); }
  }
  
  console.log(`\n\nDone: ${ok} downloaded, ${fail} failed → ${OUT_DIR}`);
  
  // Build index
  const index = list.map(p => ({
    id: p.id, name: p.name, type: p.type,
    categories: (p.categories||[]).map(c=>c.value),
    technologies: p.technologies?.value || '',
    creator: p.creator?.value || '',
    short_description: p.short_description || '',
    certified: p.certified || false,
  }));
  fs.writeFileSync(path.join(OUT_DIR, 'index.json'), JSON.stringify(index, null, 2));

  await browser.close();
}
main().catch(console.error);
