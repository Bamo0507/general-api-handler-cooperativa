use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Utc};
use general_api::endpoints::handlers::configs::schema::GeneralContext;
use general_api::endpoints::handlers::graphql::quota::QuotaQuery;
use general_api::models::graphql::{Quota, QuotaType};
use general_api::repos::auth::utils::hashing_composite_key;
use general_api::test_sync::REDIS_TEST_LOCK;
use redis::Commands;

use super::common::{create_test_context, TestRedisGuard};

#[derive(Clone, Debug)]
struct ExpectedQuota {
    due_date: String,
    identifier: String,
    amount: f64,
    user_id: String,
}

fn spanish_month_name(month: u32) -> &'static str {
    match month {
        1 => "Enero",
        2 => "Febrero",
        3 => "Marzo",
        4 => "Abril",
        5 => "Mayo",
        6 => "Junio",
        7 => "Julio",
        8 => "Agosto",
        9 => "Septiembre",
        10 => "Octubre",
        11 => "Noviembre",
        12 => "Diciembre",
        _ => "Mes",
    }
}

fn identifier_for(name: &str, date: &NaiveDate) -> String {
    let month = spanish_month_name(date.month());
    format!("{} - {} {}", name, month, date.year())
}

fn register_affiliate_metadata(
    context: &GeneralContext,
    guard: &mut TestRedisGuard,
    access_token: &str,
    affiliate_key_value: &str,
    complete_name: &str,
) {
    let mut conn = context.pool.get().expect("No se pudo obtener conexión a Redis");
    let affiliate_key = format!("users:{}:affiliate_key", access_token);
    let complete_name_key = format!("users:{}:complete_name", access_token);
    let _: () = conn
        .set(&affiliate_key, affiliate_key_value)
        .expect("No se pudo registrar affiliate_key de prueba");
    let _: () = conn
        .set(&complete_name_key, complete_name)
        .expect("No se pudo registrar complete_name de prueba");
    guard.register_key(affiliate_key);
    guard.register_key(complete_name_key);
}

fn register_quota_key(guard: &mut TestRedisGuard, access_token: &str, quota: &Quota) {
    let exp_date = quota
        .exp_date
        .as_ref()
        .expect("Las quotas de prueba deben tener fecha");
    let access_token_owned = access_token.to_string();
    let db_access_token = hashing_composite_key(&[&access_token_owned]);
    let key = format!("users:{}:quotas_afiliado:{}", db_access_token, exp_date);
    guard.register_key(key);
}

#[test]
fn test_get_monthly_affiliate_quota_returns_pending_formatted() {
    // Serializa el acceso a Redis compartido entre pruebas.
    let _lock = REDIS_TEST_LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .expect("No se pudo adquirir el lock de pruebas");

    let context = create_test_context();
    let mut guard = TestRedisGuard::new(context.pool.clone());
    let repo = context.quota_repo();

    let today = Utc::now().date_naive();
    let current_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .expect("Mes actual inválido");
    let previous_month = if today.month() == 1 {
        NaiveDate::from_ymd_opt(today.year() - 1, 12, 1).expect("Mes anterior inválido")
    } else {
        NaiveDate::from_ymd_opt(today.year(), today.month() - 1, 1)
            .expect("Mes anterior inválido")
    };

    let affiliates = vec![
        (
            "quota_afiliado_token_juan",
            "Juan Perez",
            "AFF-JP",
            vec![(current_month, 250.0_f64), (previous_month, 275.0_f64)],
        ),
        (
            "quota_afiliado_token_maria",
            "Maria Gomez",
            "AFF-MG",
            vec![(previous_month, 180.0_f64)],
        ),
    ];

    let mut expected: HashMap<String, Vec<ExpectedQuota>> = HashMap::new();

    for (access_token, name, affiliate_key_value, due_dates) in &affiliates {
        register_affiliate_metadata(
            &context,
            &mut guard,
            access_token,
            affiliate_key_value,
            name,
        );

        for (due_date, amount) in due_dates.iter() {
            let due_date_str = due_date.format("%Y-%m-%d").to_string();
            let quota = Quota {
                user_id: (*access_token).to_string(),
                amount: *amount,
                exp_date: Some(due_date_str.clone()),
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
                .save_quota((*access_token).to_string(), &quota)
                .expect("No se pudo guardar la quota de prueba");

            register_quota_key(&mut guard, access_token, &quota);

            expected
                .entry((*name).to_string())
                .or_default()
                .push(ExpectedQuota {
                    due_date: due_date_str,
                    identifier: identifier_for(name, due_date),
                    amount: *amount,
                    user_id: (*access_token).to_string(),
                });
        }
    }

    let expected_total: usize = expected.values().map(|entries| entries.len()).sum();

    let result = futures::executor::block_on(QuotaQuery::get_monthly_affiliate_quota(
        &context,
        "TEST_REQUESTING_TOKEN".to_string(),
    ))
    .expect("La query de quotas de afiliado falló");

    let mut matched = 0_usize;

    for quota in result {
        let Some(name) = quota.nombre_usuario.clone() else {
            continue;
        };

        let Some(expected_entries) = expected.get_mut(&name) else {
            continue;
        };

        let exp_date = quota
            .exp_date
            .clone()
            .expect("Las cuotas devueltas deben incluir exp_date");
        let identifier = quota
            .identifier
            .clone()
            .expect("Las cuotas devueltas deben incluir identifier");

        if let Some(position) = expected_entries.iter().position(|candidate| {
            candidate.due_date == exp_date
                && candidate.identifier == identifier
                && (candidate.amount - quota.amount).abs() < f64::EPSILON
        }) {
            let expected_entry = expected_entries.remove(position);
            matched += 1;

            assert_eq!(quota.quota_type, QuotaType::Afiliado);
            assert_eq!(quota.user_id, expected_entry.user_id);
            assert_eq!(quota.payed, Some(false));
            assert_eq!(quota.is_extraordinary, Some(false));
            assert_eq!(quota.monto_pagado, Some(0.0));
            assert_eq!(quota.multa, Some(0.0));
        }
    }

    for (name, remaining) in &expected {
        assert!(remaining.is_empty(), "Faltan quotas esperadas para {}", name);
    }

    assert_eq!(matched, expected_total, "No se validaron todas las quotas esperadas");
}
