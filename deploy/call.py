"""Call one Ghostkeys MCP tool; save any returned screenshot locally."""
import base64,json,pathlib,sys,urllib.request
root=pathlib.Path(__file__).resolve().parent.parent
config=json.loads((root/'.private/mcp-connection.json').read_text())
name=sys.argv[1]
args=json.loads(sys.argv[2]) if len(sys.argv)>2 else {}
req=urllib.request.Request(config['url'],data=json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':name,'arguments':args}}).encode(),headers={**config['headers'],'Content-Type':'application/json','Accept':'application/json, text/event-stream'})
with urllib.request.urlopen(req,timeout=60) as r:result=json.load(r)
for content in result.get('result',{}).get('content',[]):
    if content['type']=='text': print(content['text'])
    elif content['type']=='image':
        path=root/'.private/browser-check.png'
        path.write_bytes(base64.b64decode(content['data']))
        print('Screenshot saved: '+str(path))
if result.get('result',{}).get('isError') or 'error' in result: sys.exit(1)
