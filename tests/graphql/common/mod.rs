use actix_web::web::Data;
use r2d2::Pool;
use redis::{Client, Commands, JsonCommands};

use general_api::endpoints::handlers::configs::schema::GeneralContext;
use general_api::models::graphql::Payment;

pub fn create_test_context() -> GeneralContext {
    let client = Client::open("redis://127.0.0.1/").expect("No se pudo conectar a Redis");
    let pool = Pool::builder()
        .build(client)
        .expect("No se pudo crear el pool de Redis");
    //TODO: see a way to inject the s3 client in the context

    GeneralContext {
        pool: Data::new(pool),
    }
}

/// Guarda claves creadas por tests y las borra automáticamente al hacer `drop`
pub struct TestRedisGuard {
    pool: Data<Pool<Client>>,
    keys: Vec<String>,
}

impl TestRedisGuard {
    /// Crea un nuevo guard vinculado a un pool de Redis
    pub fn new(pool: Data<Pool<Client>>) -> Self {
        TestRedisGuard {
            pool,
            keys: Vec::new(),
        }
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
    use chrono::Utc;
    use general_api::models::redis::Payment as RedisPayment;
    use general_api::repos::auth::utils::hashing_composite_key;

    let pool = context.pool.clone();
    let mut con = pool.get().expect("No se pudo obtener conexión de Redis");

    // use a unique composite per helper call to avoid collisions when tests run in parallel
    // avoid deprecated timestamp_nanos, build a unique string from seconds + nanos
    let now = Utc::now();
    let unique = format!("test{}_{}", now.timestamp(), now.timestamp_subsec_nanos());
    let composite_key = hashing_composite_key(&[&unique]);
    let redis_key = format!("users:{}:payments:{}", composite_key, payment.id);

    let redis_payment = RedisPayment {
        date_created: payment.payment_date.clone(),
        account_number: payment.account_num.clone(),
        total_amount: payment.total_amount,
        name: payment.name.clone(),
        comments: payment.commentary.clone(),
        comprobante_bucket: payment.photo_path.clone(),
        ticket_number: payment.ticket_num.clone(),
        status: payment.state.as_str().to_string(),
        being_payed: vec![],
    };

    let _: redis::RedisResult<()> = con.json_set(&redis_key, "$", &redis_payment);
    redis_key
}

#[cfg(test)]
mod integration {
    use super::*;
    use general_api::models::graphql::{Payment, PaymentStatus};
    use redis::Commands;

    #[test]
    fn test_guard_borra_solo_claves_registradas() {
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
            photo_path: "url1".to_string(),
            state: PaymentStatus::Accepted,
            being_payed: vec![],
            presented_by_name: "N/A".to_string(),
        };

        let payment2 = Payment {
            id: "test_guard_pago2".to_string(),
            name: "Test2".to_string(),
            total_amount: 20.0,
            payment_date: "2025-10-14".to_string(),
            ticket_num: "T2".to_string(),
            account_num: "ACC2".to_string(),
            commentary: Some("Pago guard 2".to_string()),
            photo_path: "url2".to_string(),
            state: PaymentStatus::Accepted,
            being_payed: vec![],
            presented_by_name: "N/A".to_string(),
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
