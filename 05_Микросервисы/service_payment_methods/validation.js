function validatePayment(data) {
  if (!Number.isInteger(data.userId) || data.userId < 1) return 'Invalid userId';
  if (!['card', 'bank_account'].includes(data.type)) return 'Invalid type';
  if (!/^\d{4}$/.test(data.lastFour || '')) return 'lastFour must contain 4 digits';
  if (!/^\d{4}-(0[1-9]|1[0-2])$/.test(data.expiresAt || '')) return 'expiresAt must be YYYY-MM';
  const [year, month] = data.expiresAt.split('-').map(Number);
  if (new Date(Date.UTC(year, month, 1)) <= new Date()) return 'Payment method expired';
  if (Object.keys(data).some(key => !['userId', 'type', 'lastFour', 'expiresAt'].includes(key)))
    return 'Unexpected fields';
  return null;
}
module.exports = { validatePayment };
