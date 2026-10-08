"""Check the actual remote MCP without printing its connection key."""
import json, pathlib, urllib.request, urllib.error
config=json.loads((pathlib.Path(__file__).resolve().parent.parent/'.private/mcp-connection.json').read_text())
def request(method,params=None):
    req=urllib.request.Request(config['url'],data=json.dumps({'jsonrpc':'2.0','id':1,'method':method,'params':params or {}}).encode(),headers={**config['headers'],'Content-Type':'application/json','Accept':'application/json, text/event-stream'})
    with urllib.request.urlopen(req,timeout=45) as response: return json.load(response)
result=request('initialize',{'protocolVersion':'2025-03-26','capabilities':{},'clientInfo':{'name':'ghostkeys-smoke','version':'1'}})
assert result['result']['serverInfo']['name']=='ghostkeys'
tools=request('tools/list')['result']['tools']
assert len(tools)==12
print('MCP initialized; 12 tools listed.')
for name in ['ghostkeys_status','ghostkeys_browser_state']:
    result=request('tools/call',{'name':name,'arguments':{}})['result']
    assert not result.get('isError'),result
    print(name+': '+result['content'][0]['text'])
for url,headers,expected in [
    ('https://ghostkeys.exe.xyz/connection',config['headers'],403),
    ('https://ghostkeys.exe.xyz/desktop/vnc.html',config['headers'],403),
]:
    try:
        urllib.request.urlopen(urllib.request.Request(url,headers=headers),timeout=20)
        raise AssertionError('MCP key was allowed owner-only access')
    except urllib.error.HTTPError as error: assert error.code==expected
print('MCP key denied desktop and connection-key access.')
