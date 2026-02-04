const { Client } = require('pg');

const client = new Client({
    host: '127.0.0.1',
    port: 18246,
    user: 'postgres',
    password: 'Y7ccKlhuwZ5JOvvPpnB9LhphltK8CJGq',
    database: 'postgres', // On se connecte à la DB par défaut 'postgres' pour en créer une autre
});

async function createDb() {
    try {
        await client.connect();
        console.log("Connecté à PostgreSQL !");

        const res = await client.query("SELECT 1 FROM pg_database WHERE datname = 'wiki'");
        if (res.rowCount === 0) {
            console.log("La base 'wiki' n'existe pas. Création en cours...");
            await client.query('CREATE DATABASE wiki');
            console.log("Base 'wiki' créée avec succès !");
        } else {
            console.log("La base 'wiki' existe déjà.");
        }
    } catch (err) {
        console.error("Erreur :", err);
    } finally {
        await client.end();
    }
}

createDb();
