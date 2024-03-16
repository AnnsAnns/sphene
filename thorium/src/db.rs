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
    pub language: Option<String>,
}

const STANDARD_LANG: &str = "en";

const STANDARD_SERVER: Server = Server {
    id: 0,
    twitter: true,
    bluesky: false,
    instagram: true,
    tiktok: false,
    language: None,
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
                    tiktok boolean not null,
                    language text
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
                let lang = row.get(5).unwrap_or("en".to_string());

                Ok(Server {
                    id: row.get(0)?,
                    twitter: row.get(1)?,
                    bluesky: row.get(2)?,
                    instagram: row.get(3)?,
                    tiktok: row.get(4)?,
                    language: Some(lang),
                })
            })
            .unwrap();

        if let Some(server) = server_iter.next() {
            server.unwrap()
        } else {
            if init {
                let mut insert_statement = self.conn.prepare(
                    
                    "INSERT INTO server (id, twitter, bluesky, instagram, tiktok, language)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)").unwrap();
                insert_statement
                    .execute(rusqlite::params![
                        id,
                        STANDARD_SERVER.twitter,
                        STANDARD_SERVER.bluesky,
                        STANDARD_SERVER.instagram,
                        STANDARD_SERVER.tiktok,
                        STANDARD_LANG.to_string()
                    ])
                    .unwrap();
            }
            STANDARD_SERVER
        }
    }

    pub fn migrate_db(&self) {
        self.conn.execute(
            "
            ALTER TABLE server
            ADD COLUMN language TEXT;
            )",
            [],
        )
        .unwrap();
    }

    pub fn update_server(&self, server: Server) {
        print!("{:?}", server);
        let mut stmt = self
            .conn
            .prepare(
                "UPDATE server 
                    SET twitter = ?1,
                        bluesky = ?2, 
                        instagram = ?3, 
                        tiktok = ?4, 
                        language = ?5 
                    WHERE id = ?6",
            )
            .unwrap();
        stmt.execute(rusqlite::params![
            server.twitter,
            server.bluesky,
            server.instagram,
            server.tiktok,
            server.language.unwrap_or(STANDARD_LANG.to_string()),
            server.id
        ])
        .unwrap();
    }
}