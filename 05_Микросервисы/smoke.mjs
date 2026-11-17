import {readFileSync, writeFileSync} from 'node:fs';
const scenarios=JSON.parse(readFileSync(new URL('./scenarios.json',import.meta.url),'utf8'));
const vars={};
const log=[];
const substitute=value=>typeof value==='string'?value.replace(/\{\{(\w+)\}\}/g,(_,key)=>vars[key]):value;
for(const scenario of scenarios) {
  const data=scenario.body && Object.fromEntries(Object.entries(scenario.body).map(([key,value])=>[
    key,key==='userId'?Number(substitute(value)):substitute(value)]));
  const response=await fetch('http://localhost:8000'+substitute(scenario.path),{
    method:scenario.method,headers:{'Content-Type':'application/json'},
    body:data?JSON.stringify(data):undefined,signal:AbortSignal.timeout(10000)
  });
  const body=await response.json();
  const headers=response.headers;
  if(response.status!==scenario.status)throw new Error(`${scenario.name}: HTTP ${response.status} ${JSON.stringify(body)}`);
  for(const [key,field] of Object.entries(scenario.save))vars[key]=body[field];
  for(const check of scenario.checks)
    if(!eval(check))throw new Error(`${scenario.name}: ${check}`);
  log.push(`PASS ${scenario.name}: HTTP ${response.status}`);
  console.log(log.at(-1));
}
writeFileSync(new URL('./test-results.txt',import.meta.url),log.join('\n'));
