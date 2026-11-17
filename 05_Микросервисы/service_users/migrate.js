const { Sequelize, DataTypes } = require('sequelize');
const db = new Sequelize(process.env.DATABASE_URL, {logging:false});
async function main() {
  const query = db.getQueryInterface();
  await db.query('CREATE TABLE IF NOT EXISTS migrations (name TEXT PRIMARY KEY)');
  const [done] = await db.query("SELECT name FROM migrations WHERE name='001_initial'");
  if (done.length) return;
  await db.transaction(async transaction => {
    const options = {transaction};
    const id = () => ({type:DataTypes.INTEGER, primaryKey:true, autoIncrement:true});
    for (const table of ['users','orders'])
      await query.createTable(table, {id:id(), data:{type:DataTypes.JSONB,allowNull:false}}, options);
    await query.createTable('payment_methods', {
      id:id(), userId:{type:DataTypes.INTEGER, allowNull:false, references:{model:'users',key:'id'}, onDelete:'CASCADE'},
      type:{type:DataTypes.STRING,allowNull:false},lastFour:{type:DataTypes.STRING(4),allowNull:false},
      expiresAt:{type:DataTypes.STRING(7),allowNull:false}
    }, options);
    await query.addIndex('payment_methods',['userId'],options);
    await db.query("CREATE INDEX orders_user_id ON orders ((data->'userId'))",options);
    await db.query("INSERT INTO migrations(name) VALUES ('001_initial')", options);
  });
}
main().then(()=>db.close()).catch(error=>{console.error(error);process.exit(1)});
