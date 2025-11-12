use envconfig::Envconfig;

#[derive(Envconfig, Debug)]
pub struct Env {
    #[envconfig(from = "HOST")]
    pub host: String,

    #[envconfig(from = "PORT")]
    pub port: u16,

    #[envconfig(from = "REDIS_URL")]
    pub redis_url: String,

    // S3 configuration (optional)
    #[envconfig(from = "BUCKET_NAME", default = "")]
    pub bucket_name: String,

    #[envconfig(from = "AWS_ACCESS_KEY_ID", default = "")]
    pub aws_access_key_id: String,

    #[envconfig(from = "AWS_SECRET_ACCESS_KEY", default = "")]
    pub aws_secret_access_key: String,

    #[envconfig(from = "AWS_REGION", default = "us-east-1")]
    pub aws_region: String,
}

impl Env {
    pub fn env_init() -> Env {
        Env::init_from_env().unwrap()
    }
}
