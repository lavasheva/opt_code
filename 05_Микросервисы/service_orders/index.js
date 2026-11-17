const express = require('express');
const {Sequelize,DataTypes} = require('sequelize');
const app=express();
app.use(express.json({limit:'32kb'}));
const db=new Sequelize(process.env.DATABASE_URL,{logging:false});
const Record=db.define('Record',{data:{type:DataTypes.JSONB,allowNull:false}}, {tableName:'orders',timestamps:false});
const run=fn=>(req,res,next)=>Promise.resolve(fn(req,res)).catch(next);
const view=row=>({...row.data,id:row.id});
app.get('/orders/health',(req,res)=>res.json({status:'OK',service:'Orders Service',timestamp:new Date().toISOString()}));
app.get('/orders/status',(req,res)=>res.json({status:'Orders service is running'}));
app.get('/orders',run(async(req,res)=>{
 const where=req.query.userId?{data:{userId:Number(req.query.userId)}}:{};
 res.json((await Record.findAll({where,order:[['id','ASC']]})).map(view));
}));
app.get('/orders/:id',run(async(req,res)=>{
 const row=await Record.findByPk(req.params.id);
 res.status(row?200:404).json(row?view(row):{error:'Order not found'});
}));
app.post('/orders',run(async(req,res)=>{
 const data={...req.body};delete data.id;
 res.status(201).json(view(await Record.create({data})));
}));
app.put('/orders/:id',run(async(req,res)=>{
 const row=await Record.findByPk(req.params.id);
 if(!row)return res.status(404).json({error:'Order not found'});
 const data={...req.body};delete data.id;
 await row.update({data});res.json(view(row));
}));
app.delete('/orders/:id',run(async(req,res)=>{
 const row=await Record.findByPk(req.params.id);
 if(!row)return res.status(404).json({error:'Order not found'});
 const deleted=view(row);await row.destroy();
 res.json({message:'Order deleted',deletedOrder:deleted});
}));
app.use((error,req,res,next)=>{
 console.error(error.message);
 res.status(error.name==='SequelizeDatabaseError'?400:500).json({error:'Invalid request or database error'});
});
db.authenticate().then(()=>app.listen(8000,'0.0.0.0'));
