const assert = require('node:assert/strict');
const { validatePayment } = require('./validation');
const valid = { userId: 1, type: 'card', lastFour: '1234', expiresAt: '2099-12' };
assert.equal(validatePayment(valid), null);
for (const patch of [{userId: 0}, {type: 'cash'}, {lastFour: '12345'}, {expiresAt: '2020-01'}, {expiresAt: '2099-13'}, {cvv:'123'}])
  assert.ok(validatePayment({...valid, ...patch}));
console.log('PASS: 7 validation checks');
