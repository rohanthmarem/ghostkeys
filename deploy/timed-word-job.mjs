// Runs on the VM. All writes use the actual Ghostkeys MCP and desktop engine.
import {readFile,writeFile,mkdir,rename} from 'node:fs/promises';
import path from 'node:path';
import {setTimeout as sleep} from 'node:timers/promises';

const [sourceFile,stateFile] = process.argv.slice(2);
if (!sourceFile || !stateFile) throw new Error('Pass source.json and state.json');
const source = JSON.parse(await readFile(sourceFile,'utf8'));
let state = JSON.parse(await readFile(stateFile,'utf8'));
const dir=path.dirname(stateFile);
await mkdir(dir,{recursive:true,mode:0o700});
async function save() {
  state.updatedAt=new Date().toISOString();
  await writeFile(stateFile+'.pending',JSON.stringify(state,null,2),{mode:0o600});
  await rename(stateFile+'.pending',stateFile);
}
async function call(name,args={}) {
  const response=await fetch('http://127.0.0.1:8790/mcp',{method:'POST',headers:{'X-Exedev-Email':'rohanth.marem@gmail.com','Content-Type':'application/json','Accept':'application/json, text/event-stream'},body:JSON.stringify({jsonrpc:'2.0',id:1,method:'tools/call',params:{name,arguments:args}}),signal:AbortSignal.timeout(45000)});
  const result=await response.json();
  if (!response.ok || result.error || result.result?.isError) throw new Error(result.result?.content?.[0]?.text || result.error?.message || 'MCP request failed');
  return result.result;
}
const value = result => JSON.parse(result.content.find(c=>c.type==='text').text);
async function screenshot(label) {
  const result = await call('ghostkeys_read_tab',{tabId:state.tabId});
  const image = result.content.find(c=>c.type==='image');
  if (image) await writeFile(path.join(dir,label+'.png'),Buffer.from(image.data,'base64'),{mode:0o600});
  return value(result);
}
const normalize = s => s.replace(/[‘’]/g,"'").replace(/[“”]/g,'"').replace(/\s+/g,' ').trim();
try {
  if (!['created','waiting'].includes(state.phase)) throw new Error('This saved job needs review before it can resume. It will not replay a possibly typed paragraph.');
  for (let index=state.completed;index<source.paragraphs.length;index++) {
    if (index>0) {
      const due = new Date(state.nextStartAt).getTime();
      if (!Number.isFinite(due)) throw new Error('Missing saved next-paragraph time');
      while (Date.now()<due) await sleep(Math.min(due-Date.now(),30000));
    }
    const browserState=value(await call('ghostkeys_browser_state'));
    const tab=browserState.tabs.find(t=>t.id===state.tabId);
    if (!tab || !tab.url.toLowerCase().includes(state.documentId.toLowerCase())) throw new Error('The requested Word document is no longer open.');
    // Word can render only the pages near the caret. Return to the end before
    // checking the most recently completed paragraph; audit all pages at the end.
    await call('ghostkeys_focus_word_end',{tabId:state.tabId});
    const before=await screenshot('paragraph-'+(index+1)+'-before');
    const existing=normalize(before.frames.map(f=>f.text).join(' '));
    if (index>0) {
      if (!existing.includes(normalize(source.paragraphs[index-1].text.slice(-100)))) throw new Error('The last typed paragraph was not visible to the Word reader. Review the document before continuing.');
    }
    state.phase='preparing'; state.currentParagraph=index+1; await save();
    await call('ghostkeys_focus_word_end',{tabId:state.tabId});
    if (index>0) {
      await call('ghostkeys_key',{tabId:state.tabId,key:'Control+Enter'});
      state.pageBreaks=(state.pageBreaks||0)+1; await save();
    }
    const paragraph=source.paragraphs[index];
    const text=(paragraph.heading ? paragraph.heading+'\n' : '')+paragraph.text;
    await call('ghostkeys_load_text',{text});
    state.phase='typing'; state.paragraphStartedAt=new Date().toISOString(); await save();
    const start=value(await call('ghostkeys_start',{tabId:state.tabId,actionId:state.jobId+'-paragraph-'+(index+1),wpm:source.wpm}));
    if (start.status !== 'countdown') throw new Error('Typing was not started; review the recorded action before continuing.');
    const deadline=Date.now()+30*60000;
    let result;
    do {
      await sleep(1000);
      result=value(await call('ghostkeys_status'));
      if (['error','ready','idle'].includes(result.status)) throw new Error('Typing stopped before paragraph completion.');
    } while (result.status!=='done' && Date.now()<deadline);
    if (result.status!=='done' || result.progress.percent!==100) throw new Error('Paragraph typing did not complete.');
    const completedAt=new Date().toISOString();
    const after=await screenshot('paragraph-'+(index+1)+'-after');
    if (!normalize(after.frames.map(f=>f.text).join(' ')).includes(normalize(paragraph.text.slice(-100)))) throw new Error('The paragraph finished typing, but its ending was not found in Word. Review before resuming.');
    state.completed=index+1;
    state.history.push({paragraph:index+1,startedAt:state.paragraphStartedAt,completedAt,characters:Array.from(text).length});
    state.lastCompletedAt=completedAt;
    state.nextStartAt=index+1<source.paragraphs.length ? new Date(new Date(completedAt).getTime()+source.gapMinutes*60000).toISOString() : null;
    state.phase=index+1<source.paragraphs.length?'waiting':'ready_for_formatting';
    await save();
    console.log('Paragraph '+(index+1)+' completed at '+completedAt+(state.nextStartAt?'; next starts no earlier than '+state.nextStartAt: '; ready for final editing and formatting'));
  }
} catch (error) {
  await call('ghostkeys_stop').catch(()=>{});
  state.phase='needs_review'; state.error=error.message; await save();
  console.error(error.message); process.exitCode=1;
}
