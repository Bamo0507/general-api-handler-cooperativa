// Tests para la query de quotas por id de usuario
// SCRUM-257: Validar que los datos coincidan con la tabla quotas y los tipos sean correctos

use actix_web::web::Data;
use general_api::models::graphql::{Quota, QuotaType};
use general_api::repos::auth::utils::hashing_composite_key;
use general_api::repos::graphql::quota::QuotaRepo;
use r2d2::Pool;
use redis::{Client, Commands};

/// Guard sencillo para registrar claves creadas durante los tests y eliminarlas al terminar.
struct QuotaTestRedisGuard {
    pool: Data<Pool<Client>>,
    keys: Vec<String>,
}

impl QuotaTestRedisGuard {
    fn new(pool: Data<Pool<Client>>) -> Self {
        QuotaTestRedisGuard { pool, keys: Vec::new() }
    }

    fn register_key(&mut self, key: String) {
        self.keys.push(key);
    }
}

impl Drop for QuotaTestRedisGuard {
    fn drop(&mut self) {
        if let Ok(mut conn) = self.pool.get() {
            for key in &self.keys {
                let _: () = conn.del(key).unwrap_or(());
            }
        }
    }
}

fn create_quota_repo() -> QuotaRepo {
    let redis_url = std::env::var("REDIS_TEST_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = Client::open(redis_url).expect("No se pudo conectar a Redis");
    let pool = Pool::builder().build(client).expect("No se pudo crear el pool de Redis");
    QuotaRepo { pool: Data::new(pool) }
}

fn register_saved_quota(guard: &mut QuotaTestRedisGuard, access_token: &str, quota: &Quota) {
    if let Some(key) = quota_redis_key(access_token, quota) {
        guard.register_key(key);
    }
}

fn quota_redis_key(access_token: &str, quota: &Quota) -> Option<String> {
    let composite = hashing_composite_key(&[&access_token.to_string()]);
    match quota.quota_type {
        QuotaType::Prestamo => {
            let loan_id = quota.loan_id.as_ref()?;
            let exp_date = quota.exp_date.as_ref()?;
            Some(format!("users:{}:loans:{}:quotas:{}", composite, loan_id, exp_date))
        }
        QuotaType::Afiliado => {
            let exp_date = quota.exp_date.as_ref()?;
            Some(format!("users:{}:quotas_afiliado:{}", composite, exp_date))
        }
    }
}

#[test]
fn test_consulta_basica_quotas_pendientes() {
    let repo = create_quota_repo();
    let mut guard = QuotaTestRedisGuard::new(repo.pool.clone());

    let access_token = "test_token_123".to_string();
    let quota = Quota {
        user_id: access_token.clone(),
        amount: 1000.0,
        exp_date: Some("2025-09-01".to_string()),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan_abc".to_string()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(1),
        nombre_prestamo: Some("Prestamo ABC".to_string()),
        nombre_usuario: Some("Usuario Test".to_string()),
        identifier: None,
    };

    repo
        .save_quota(access_token.clone(), &quota)
        .expect("No se pudo guardar la quota de prueba");
    register_saved_quota(&mut guard, &access_token, &quota);

    let result = repo
        .get_pending_quotas(access_token.clone())
        .expect("La consulta de quotas pendientes falló");

    assert_eq!(result.len(), 1, "Debe retornar una quota pendiente");
    let returned = &result[0];
    assert_eq!(returned.user_id, quota.user_id);
    assert_eq!(returned.amount, quota.amount);
    assert_eq!(returned.exp_date, quota.exp_date);
    assert_eq!(returned.monto_pagado, quota.monto_pagado);
    assert_eq!(returned.multa, quota.multa);
    assert_eq!(returned.pay_by, quota.pay_by);
    assert_eq!(returned.quota_type, QuotaType::Prestamo);
    assert_eq!(returned.loan_id, quota.loan_id);
    assert_eq!(returned.quota_number, quota.quota_number);
}

#[test]
fn test_filtrado_por_tipo_quota() {
    let repo = create_quota_repo();
    let mut guard = QuotaTestRedisGuard::new(repo.pool.clone());

    let access_token = "test_token_tipo".to_string();
    let quota_prestamo = Quota {
        user_id: access_token.clone(),
        amount: 500.0,
        exp_date: Some("2025-09-10".to_string()),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Prestamo,
        loan_id: Some("loan_xyz".to_string()),
        is_extraordinary: None,
        payed: Some(false),
        quota_number: Some(1),
        nombre_prestamo: Some("Prestamo XYZ".to_string()),
        nombre_usuario: None,
        identifier: None,
    };

    let quota_afiliado = Quota {
        user_id: access_token.clone(),
        amount: 200.0,
        exp_date: Some("2025-09-15".to_string()),
        monto_pagado: Some(0.0),
        multa: Some(0.0),
        pay_by: None,
        quota_type: QuotaType::Afiliado,
        loan_id: None,
        is_extraordinary: Some(false),
        payed: Some(false),
        quota_number: None,
        nombre_prestamo: None,
        nombre_usuario: None,
        identifier: None,
    };

    repo
        .save_quota(access_token.clone(), &quota_prestamo)
        .expect("No se pudo guardar la quota de préstamo");
    repo
        .save_quota(access_token.clone(), &quota_afiliado)
        .expect("No se pudo guardar la quota de afiliado");

    register_saved_quota(&mut guard, &access_token, &quota_prestamo);
    register_saved_quota(&mut guard, &access_token, &quota_afiliado);

    let result = repo
        .get_pending_quotas(access_token.clone())
        .expect("La consulta de quotas pendientes falló");

    assert_eq!(result.len(), 2, "Debe retornar dos quotas pendientes");

    let mut found_prestamo = false;
    let mut found_afiliado = false;

    for quota in result {
        match quota.quota_type {
            QuotaType::Prestamo => {
                assert_eq!(quota.loan_id.as_deref(), Some("loan_xyz"));
                assert_eq!(quota.quota_number, Some(1));
                found_prestamo = true;
            }
            QuotaType::Afiliado => {
                assert_eq!(quota.is_extraordinary, Some(false));
                assert!(quota.loan_id.is_none());
                found_afiliado = true;
            }
        }
    }

    assert!(found_prestamo, "No se encontró quota tipo Préstamo");
    assert!(found_afiliado, "No se encontró quota tipo Afiliado");
}
