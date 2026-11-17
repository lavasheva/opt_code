const express = require('express');
const axios = require('axios');
const CircuitBreaker = require('opossum');
const {createClient} = require('redis');
const app = express();
app.use(express.json({limit:'32kb'}));
const cache = createClient({url:process.env.REDIS_URL, socket:{reconnectStrategy:()=>1000}});
cache.on('error', error=>console.error('Redis:',error.message));
cache.connect().catch(console.error);
const breakers = {};
for (const [name,url] of Object.entries({users:process.env.USERS_URL,orders:process.env.ORDERS_URL,'payment-methods':process.env.PAYMENT_URL})) {
  breakers[name] = new CircuitBreaker(async (method,path,data)=>{
    const response = await axios({method,url:url+path,data,timeout:2000,validateStatus:s=>s<500});
    return {status:response.status,body:response.data};
  }, {timeout:3000,errorThresholdPercentage:50,resetTimeout:5000,volumeThreshold:3});
}
const run = fn=>(req,res,next)=>Promise.resolve(fn(req,res)).catch(next);
app.get('/users/:id/details',run(async(req,res)=>{
  const [user,orders] = await Promise.all([
    breakers.users.fire('GET',`/users/${req.params.id}`),
    breakers.orders.fire('GET',`/orders?userId=${encodeURIComponent(req.params.id)}`)
  ]);
  if(user.status!==200)return res.status(user.status).json(user.body);
  if(orders.status!==200)return res.status(orders.status).json(orders.body);
  res.json({user:user.body,orders:orders.body});
}));
app.get('/status',(req,res)=>res.json({status:'API Gateway is running'}));
app.get('/health',(req,res)=>res.json({status:'API Gateway is running',circuits:Object.fromEntries(
  Object.entries(breakers).map(([key,value])=>[key,{open:value.opened,stats:value.stats}]))}));
app.use(run(async(req,res)=>{
  const name=req.path.split('/')[1];
  if(!breakers[name])return res.status(404).json({error:'Route not found'});
  const individual = new RegExp(`^/${name}/[0-9]+$`).test(req.path);
  let version='0',key;
  if(cache.isReady) {
    try { version=await cache.get('version:'+name)||'0'; } catch(error) { console.error(error.message); }
  }
  key=`${name}:${version}:${req.originalUrl}`;
  if(req.method==='GET'&&individual&&cache.isReady) {
    try {
      const hit=await cache.get(key);
      if(hit){res.set('X-Cache','HIT');return res.json(JSON.parse(hit));}
    } catch(error){console.error(error.message);}
  }
  const result=await breakers[name].fire(req.method,req.originalUrl,req.body);
  if(cache.isReady) {
    try {
      if(req.method==='GET'&&individual&&result.status===200)
        await cache.set(key,JSON.stringify(result.body),{EX:30});
      if(['POST','PUT','DELETE'].includes(req.method)&&result.status<300) {
        await cache.incr('version:'+name);
        if(name==='users')await cache.incr('version:payment-methods');
      }
    } catch(error){console.error(error.message);}
  }
  res.set('X-Cache','MISS');
  res.status(result.status).json(result.body);
}));
app.use((error,req,res,next)=>{
  console.error(error.message);
  res.status(error.type==='entity.parse.failed'?400:503).json({error:'Request failed or service temporarily unavailable'});
});
app.listen(8000,'0.0.0.0');
