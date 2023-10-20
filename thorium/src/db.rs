use rusqlite::{Connection, Result};

pub struct DBConn {
    conn: Connection,
}

#[derive(Debug)]
pub struct Server {
    pub id: u64,
    pub twitter: bool,
    pub bluesky: bool,
    pub instagram: bool,
    pub tiktok: bool,
}

const STANDARD_SERVER: Server = Server {
    id: 0,
    twitter: true,
    bluesky: true,
    instagram: true,
    tiktok: true,
};

impl DBConn {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("sphene.db")?;
        Ok(Self { conn })
    }

    pub fn create_new(&self) {
        self.conn
            .execute(
                "create table if not exists server (
                 id integer primary key,
                    twitter boolean not null,
                    bluesky boolean not null,
                    instagram boolean not null,
                    tiktok boolean not null
             )",
                [],
            )
            .unwrap();
    }

    pub fn get_server(&self, id: u64, init: bool) -> Server {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM server WHERE id = ?1")
            .unwrap();
        let mut server_iter = stmt
            .query_map([id], |row| {
                Ok(Server {
                    id: row.get(0)?,
                    twitter: row.get(1)?,
                    bluesky: row.get(2)?,
                    instagram: row.get(3)?,
                    tiktok: row.get(4)?,
                })
            })
            .unwrap();

        if let Some(server) = server_iter.next() {
            server.unwrap()
        } else {
            if init {
                let mut insert_statement = self.conn.prepare("INSERT INTO server (id, twitter, bluesky, instagram, tiktok) VALUES (?1, ?2, ?3, ?4, ?5)").unwrap();
                insert_statement
                    .execute(rusqlite::params![
                        id,
                        STANDARD_SERVER.twitter,
                        STANDARD_SERVER.bluesky,
                        STANDARD_SERVER.instagram,
                        STANDARD_SERVER.tiktok
                    ])
                    .unwrap();
            }
            STANDARD_SERVER
        }
    }

    pub fn update_server(&self, server: Server) {
        print!("{:?}", server);
        let mut stmt = self
            .conn
            .prepare("UPDATE server SET twitter = ?1, bluesky = ?2, instagram = ?3, tiktok = ?4 WHERE id = ?5")
            .unwrap();
        stmt.execute(rusqlite::params![
            server.twitter,
            server.bluesky,
            server.instagram,
            server.tiktok,
            server.id
        ])
        .unwrap();
    }
}
