// Switch to admin database first
db = db.getSiblingDB('admin');

// Create application database
const dbName = process.env.MONGO_DATABASE;
db = db.getSiblingDB(dbName);

// Create application user with proper authentication
db.createUser({
    user: process.env.MONGO_APP_USERNAME,
    pwd: process.env.MONGO_APP_PASSWORD,
    roles: [
        {
            role: 'readWrite',
            db: dbName
        }
    ]
});

// Create collections with indexes
db.createCollection('users');
db.users.createIndex({ "email": 1 }, { unique: true });
db.users.createIndex({ "deleted_at": 1 }); 