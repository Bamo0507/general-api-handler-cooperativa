use actix_web::web::Data;
use r2d2::Pool;
use redis::{Client, Commands, JsonCommands};

use general_api::endpoints::handlers::configs::schema::GeneralContext;
use general_api::models::graphql::Payment;


pub fn create_test_context() -> GeneralContext {
    let client = Client::open("redis://127.0.0.1/").expect("No se pudo conectar a Redis");
    let pool = Pool::builder().build(client).expect("No se pudo crear el pool de Redis");
    GeneralContext { pool: Data::new(pool) }
}



/// Guarda claves creadas por tests y las borra automáticamente al hacer `drop`
pub struct TestRedisGuard {
    pool: Data<Pool<Client>>,
    keys: Vec<String>,
}

impl TestRedisGuard {
    /// Crea un nuevo guard vinculado a un pool de Redis
    pub fn new(pool: Data<Pool<Client>>) -> Self {
        TestRedisGuard { pool, keys: Vec::new() }
    }

    /// Registra una clave para su posterior eliminación
    pub fn register_key(&mut self, key: String) {
        self.keys.push(key);
    }
}

impl Drop for TestRedisGuard {
    /// Elimina todas las claves registradas cuando el guard sale de alcance
    fn drop(&mut self) {
        if let Ok(mut con) = self.pool.get() {
            for key in &self.keys {
                let _: () = con.del(key).unwrap_or(());
            }
        }
    }
}



/// Inserta un pago en Redis y devuelve la clave usada
pub fn insert_payment_helper_and_return(context: &GeneralContext, payment: &Payment) -> String {
    use general_api::models::redis::Payment as RedisPayment;
    use general_api::repos::auth::utils::hashing_composite_key;

    let pool = context.pool.clone();
    let mut con = pool.get().expect("No se pudo obtener conexión de Redis");

    let composite_key = hashing_composite_key(&[&String::from("all")]);
    let redis_key = format!("users:{}:payments:{}", composite_key, payment.id);

    let redis_payment = RedisPayment {
        date_created: payment.payment_date.clone(),
        account_number: payment.account_num.clone(),
        total_amount: payment.total_amount,
        name: payment.name.clone(),
        comments: payment.commentary.clone(),
        comprobante_bucket: payment.photo.clone(),
        ticket_number: payment.ticket_num.clone(),
        status: payment.state.as_str().to_string(),
        being_payed: vec![],
    };

    let _: redis::RedisResult<()> = con.json_set(&redis_key, "$", &redis_payment);
    redis_key
}

/// Inserta una multa en Redis y devuelve la clave usada. Permite probar FineQuery.
#[allow(dead_code)]
pub fn insert_fine_helper_and_return(
    context: &GeneralContext,
    access_token: &str,
    fine: &general_api::models::graphql::Fine,
) -> String {
    use general_api::models::redis::Fine as RedisFine;
    use general_api::repos::auth::utils::hashing_composite_key;

    let pool = context.pool.clone();
    let mut con = pool.get().expect("No se pudo obtener conexión de Redis");

    let composite_key = hashing_composite_key(&[&access_token.to_string()]);
    let redis_key = format!("users:{}:fines:{}", composite_key, fine.id);

    let redis_fine = RedisFine {
        amount: fine.amount as f32,
        motive: fine.reason.clone(),
    status: fine.status.to_string(),
    };

    let _: redis::RedisResult<()> = con.json_set(&redis_key, "$", &redis_fine);
    redis_key
}


#[cfg(test)]
mod integration {
    use super::*;
    use general_api::models::graphql::{Payment, PaymentStatus};
    use redis::Commands;

    #[test]
    fn test_guard_borra_solo_claves_registradas() {
        let _redis_guard = general_api::test_sync::REDIS_TEST_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap();
        let context = create_test_context();
        let mut guard = TestRedisGuard::new(context.pool.clone());

        // Insertar dos pagos y registrar solo uno
        let payment1 = Payment {
            id: "test_guard_pago1".to_string(),
            name: "Test1".to_string(),
            total_amount: 10.0,
            payment_date: "2025-10-14".to_string(),
            ticket_num: "T1".to_string(),
            account_num: "ACC1".to_string(),
            commentary: Some("Pago guard 1".to_string()),
            photo: "url1".to_string(),
            state: PaymentStatus::Accepted,
        };

        let payment2 = Payment {
            id: "test_guard_pago2".to_string(),
            name: "Test2".to_string(),
            total_amount: 20.0,
            payment_date: "2025-10-14".to_string(),
            ticket_num: "T2".to_string(),
            account_num: "ACC2".to_string(),
            commentary: Some("Pago guard 2".to_string()),
            photo: "url2".to_string(),
            state: PaymentStatus::Accepted,
        };

        let key1 = insert_payment_helper_and_return(&context, &payment1);
        let key2 = insert_payment_helper_and_return(&context, &payment2);
        guard.register_key(key1.clone());

        // Verificar que ambas claves existen antes del drop
        let mut con = context.pool.get().unwrap();
        assert!(
            con.exists::<_, bool>(&key1).unwrap(),
            "La clave 1 debe existir antes de drop"
        );
        assert!(
            con.exists::<_, bool>(&key2).unwrap(),
            "La clave 2 debe existir antes de drop"
        );

        // Drop explícito del guard para limpiar solo la clave registrada
        drop(guard);

        // Verificar que solo la clave registrada fue borrada
        assert!(
            !con.exists::<_, bool>(&key1).unwrap(),
            "La clave 1 debe haber sido borrada por el guard"
        );
        assert!(
            con.exists::<_, bool>(&key2).unwrap(),
            "La clave 2 NO debe haber sido borrada por el guard"
        );

        // Limpieza manual de la clave 2 para no dejar basura en Redis
        let _: () = con.del(&key2).unwrap_or(());
    }
}
