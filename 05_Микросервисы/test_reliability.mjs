// Проверка на работающей системе. Сервис orders кратко останавливается и запускается обратно.
import {spawnSync} from 'node:child_process';
import {writeFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';

const directory=fileURLToPath(new URL('.',import.meta.url));
const docker=process.platform==='win32'?'C:\\Program Files\\Docker\\Docker\\resources\\bin\\docker.exe':'docker';
const log=[];
const pause=milliseconds=>new Promise(resolve=>setTimeout(resolve,milliseconds));
function compose(...args) {
  const result=spawnSync(docker,['compose',...args],{cwd:directory,encoding:'utf8'});
  if(result.status!==0)throw new Error(result.stderr||result.error);
}
async function get(path) {
  return fetch('http://localhost:8000'+path,{signal:AbortSignal.timeout(7000)});
}
function check(condition,message) {
  if(!condition)throw new Error(message);
  log.push('PASS '+message);
  console.log(log.at(-1));
}
await pause(11000); // Истекает окно статистики предыдущих успешных запросов.
try {
  compose('stop','service_orders');
  for(let attempt=0;attempt<4;attempt++) {
    const response=await get('/orders');
    check(response.status===503,'Unavailable orders return 503, attempt '+(attempt+1));
  }
  const health=await (await get('/health')).json();
  check(health.circuits.orders.open===true,'Orders circuit is open');
  check((await get('/users')).status===200,'Users remain available');
} finally {
  compose('start','service_orders');
}
await pause(6000);
check((await get('/orders')).status===200,'Orders recover after restart');
const health=await (await get('/health')).json();
check(health.circuits.orders.open===false,'Orders circuit closes after recovery');
writeFileSync(new URL('./reliability-results.txt',import.meta.url),log.join('\n'));
