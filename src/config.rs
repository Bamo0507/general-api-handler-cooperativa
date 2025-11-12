use envconfig::Envconfig;

#[derive(Envconfig, Debug)]
pub struct Env {
    #[envconfig(from = "HOST")]
    pub host: String,

    #[envconfig(from = "PORT")]
    pub port: u16,

    #[envconfig(from = "REDIS_URL")]
    pub redis_url: String,

    #[envconfig(from = "BUCKET_NAME")]
    pub bucket_name: String,

    #[envconfig(from = "AWS_ACCESS_KEY_ID")]
    pub aws_access_key_id: String,

    #[envconfig(from = "AWS_SECRET_ACCESS_KEY")]
    pub aws_secret_access_key: String,

    #[envconfig(from = "AWS_REGION")]
    pub aws_region: String,

    #[envconfig(from = "TLS_ON")]
    pub tls_on: usize,
}

impl Env {
    pub fn env_init() -> Env {
        Env::init_from_env().unwrap()
    }
}
