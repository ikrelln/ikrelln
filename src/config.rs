use clap::{App, Arg};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub db_nb_connection: usize,
    pub db_url: String,
}
impl Config {
    pub fn load() -> Config {
        let version: String = format!("v{}", ::build_info::BUILD_INFO.version);

        // configuration
        let matches = App::new("i'Krelln")
            .version(version.as_str())
            .about("Start i'Krelln server")
            .arg(
                Arg::with_name("host")
                    .long("host")
                    .takes_value(true)
                    .value_name("HOST")
                    .default_value("0.0.0.0")
                    .env("HOST")
                    .help("Listen on the specified host"),
            )
            .arg(
                Arg::with_name("port")
                    .short("p")
                    .long("port")
                    .takes_value(true)
                    .value_name("PORT")
                    .default_value("8080")
                    .env("PORT")
                    .help("Listen to the specified port"),
            )
            .arg(
                Arg::with_name("nb_connection")
                    .long("nb-connection")
                    .takes_value(true)
                    .value_name("NB_CONNECTION")
                    .default_value("5")
                    .env("NB_CONNECTION")
                    .help("Open this number of connections to the DB"),
            )
            .arg(
                Arg::with_name("database_url")
                    .long("db-url")
                    .takes_value(true)
                    .value_name("DATABASE_URL")
                    .env("DATABASE_URL")
                    .help("Url to the DB"),
            )
            .get_matches();

        let host = matches.value_of("host").unwrap().to_string();

        let port = matches
            .value_of("port")
            .and_then(|it| it.parse::<u16>().ok())
            .unwrap();

        let db_nb_connection = matches
            .value_of("nb_connection")
            .and_then(|it| it.parse::<usize>().ok())
            .unwrap();

        let db_url = matches
            .value_of("database_url")
            .expect("missing DATABASE_URL parameter")
            .to_string();

        Config {
            host: host,
            port: port,
            db_nb_connection: db_nb_connection,
            db_url: db_url,
        }
    }
}
