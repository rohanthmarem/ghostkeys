// Run on the VM. Exercises the real desktop engine against a disposable browser editor.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {chromium} from 'playwright';
const env = Object.fromEntries((await readFile('/home/exedev/.config/ghostkeys/service.env','utf8')).trim().split('\n').map(line=>{const i=line.indexOf('=');return [line.slice(0,i),line.slice(i+1)];}));
async function command(action,extra={}) {
  const r = await fetch('http://127.0.0.1:8789/command',{method:'POST',headers:{Authorization:'Bearer '+env.GHOSTKEYS_API_TOKEN,'Content-Type':'application/json'},body:JSON.stringify({action,...extra})});
  const v = await r.json(); if (!r.ok) throw new Error(v.error); return v;
}
const browser = await chromium.connectOverCDP('http://127.0.0.1:9222');
const page = await browser.contexts()[0].newPage();
const text = 'Ghostkeys VM test: café, naïve, and Unicode.\nSecond paragraph.';
const original = (await command('status')).config;
try {
  await page.goto('file:///home/exedev/ghostkeys/deploy/typing-test.html');
  await page.bringToFront();
  await page.locator('#editor').click();
  await command('load',{text});
  await command('configure',{config:{...original,baseWpm:300,mistakeRate:0,correctionRate:1,countdownSeconds:1,wpmVariance:0,punctuationPause:0,paragraphPause:0,thinkingPauseChance:0,burstTyping:false}});
  await command('start');
  assert.equal((await command('status')).status,'countdown');
  await assert.rejects(command('start'),/already active/);
  const deadline=Date.now()+30000;
  let state;
  do {
    await new Promise(resolve=>setTimeout(resolve,250));
    state=await command('status');
  } while (!['done','error'].includes(state.status) && Date.now()<deadline);
  assert.equal(state.status,'done');
  assert.equal(await page.locator('#editor').inputValue(),text);
  assert.equal(state.progress.total,Array.from(text).length);
  assert.equal(state.progress.percent,100);
  await page.screenshot({path:'/home/exedev/ghostkeys/deploy/typing-test-result.png'});
  console.log('PASS: real Ghostkeys typed exact Unicode text and a paragraph break into Chrome; duplicate start rejected; progress reached 100%.');
  await command('load',{text:'Pause resume test '.repeat(10)});
  await command('start');
  while ((await command('status')).status==='countdown') await new Promise(resolve=>setTimeout(resolve,100));
  await command('pause');
  const paused=(await command('status')).progress.current;
  await new Promise(resolve=>setTimeout(resolve,500));
  assert.ok((await command('status')).progress.current <= paused+1);
  await command('resume');
  await new Promise(resolve=>setTimeout(resolve,300));
  assert.ok((await command('status')).progress.current > paused);
  await command('stop');
  while ((await command('status')).status!=='ready') await new Promise(resolve=>setTimeout(resolve,100));
  console.log('PASS: pause, resume, and stop controlled the actual typing session.');
  const denied=await fetch('http://127.0.0.1:8789/command',{method:'POST',headers:{'Content-Type':'application/json'},body:'{"action":"status"}'});
  assert.equal(denied.status,401);
  console.log('PASS: desktop control API rejects unauthenticated requests.');
} finally {
  await command('stop').catch(()=>{});
  await command('configure',{config:original}).catch(()=>{});
  await page.close();
  await browser.close();
}
