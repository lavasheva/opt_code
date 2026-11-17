const express = require('express');
const axios = require('axios');
const { Sequelize, DataTypes } = require('sequelize');
const { validatePayment } = require('./validation');
const app = express();
app.use(express.json({ limit: '32kb' }));
const db = new Sequelize(process.env.DATABASE_URL, { logging: false });
const PaymentMethod = db.define('PaymentMethod', {
  userId: { type: DataTypes.INTEGER, allowNull: false },
  type: { type: DataTypes.STRING, allowNull: false },
  lastFour: { type: DataTypes.STRING(4), allowNull: false },
  expiresAt: { type: DataTypes.STRING(7), allowNull: false }
}, { tableName: 'payment_methods', timestamps: false });
const run = fn => (req, res, next) => Promise.resolve(fn(req, res)).catch(next);
app.get('/payment-methods', run(async (req, res) => {
  const where = req.query.userId ? { userId: req.query.userId } : {};
  res.json(await PaymentMethod.findAll({ where, order: [['id','ASC']] }));
}));
app.get('/payment-methods/:id', run(async (req, res) => {
  const method = await PaymentMethod.findByPk(req.params.id);
  res.status(method ? 200 : 404).json(method || {error: 'Payment method not found'});
}));
async function save(req, res) {
  let method = null;
  if (req.method === 'PUT') {
    method = await PaymentMethod.findByPk(req.params.id);
    if (!method) return res.status(404).json({error: 'Payment method not found'});
  }
  const data = method ? {...method.toJSON(), ...req.body} : {...req.body};
  delete data.id;
  const error = validatePayment(data);
  if (error) return res.status(400).json({error});
  const user = await axios.get(`${process.env.USERS_URL}/users/${data.userId}`, {
    timeout: 2000, validateStatus: status => status === 200 || status === 404
  });
  if (user.status === 404) return res.status(404).json({error: 'User not found'});
  const result = method ? await method.update(data) : await PaymentMethod.create(data);
  res.status(method ? 200 : 201).json(result);
}
app.post('/payment-methods', run(save));
app.put('/payment-methods/:id', run(save));
app.delete('/payment-methods/:id', run(async (req, res) => {
  const method = await PaymentMethod.findByPk(req.params.id);
  if (!method) return res.status(404).json({error:'Payment method not found'});
  await method.destroy();
  res.json({message:'Payment method deleted', deletedPaymentMethod:method});
}));
app.use((error, req, res, next) => {
  console.error(error.message);
  res.status(503).json({error:'Payment service unavailable'});
});
db.authenticate().then(() => app.listen(8000, '0.0.0.0'));
