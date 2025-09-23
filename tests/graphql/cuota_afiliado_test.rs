use general_api::endpoints::handlers::configs::schema::GeneralContext;
use general_api::models::graphql::{Quota, TipoQuota};
use general_api::endpoints::handlers::graphql::Quota::{QuotaQuery, QuotaAfiliadoMensualResponse};
use chrono::NaiveDate;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web::Data;
    use r2d2::Pool;
    use redis::Client;

    fn setup_context() -> GeneralContext {
        // Configura un pool de Redis para pruebas (puede ser mock o real)
        let client = Client::open("redis://127.0.0.1/").unwrap();
        let pool = Pool::builder().build(client).unwrap();
        GeneralContext { pool: Data::new(pool) }
    }

    #[test]
    fn test_get_monthly_affiliate_quota() {
        let context = setup_context();
        
        // *** LIMPIAR Redis antes de la prueba ***
        {
            use redis::Commands;
            let mut conn = context.pool.get().unwrap();
            let _: () = conn.flushdb().unwrap();
        }
        
        // Limpieza y setup de datos
            // 1. Crear afiliados de prueba y sus claves en Redis
            let afiliados = vec![
                ("afiliado1", "Juan Perez"),
                ("afiliado2", "Maria Gomez"),
            ];

            // Crear claves en Redis para cada afiliado según el formato esperado por el repo
            {
                use redis::Commands;
                let mut conn = context.pool.get().unwrap();
                for (afiliado_key, nombre) in &afiliados {
                    let redis_key = format!("users:{}:affiliate_key", afiliado_key);
                    let _: () = conn.set(&redis_key, afiliado_key).unwrap();
                    let complete_name_key = format!("users:{}:complete_name", afiliado_key);
                    let _: () = conn.set(&complete_name_key, nombre).unwrap();
                }
            }

        // 2. Crear quotas pendientes para cada afiliado y guardar en Redis
        let quotas = vec![
            // Para cada afiliado, crear una Quota en dos meses distintos
            Quota {
                user_id: "afiliado1".to_string(),
                monto: 250.0,
                fecha_vencimiento: Some("2025-01-01".to_string()),
                monto_pagado: 0.0,
                multa: 0.0,
                pagada_por: None,
                tipo: TipoQuota::Afiliado,
                loan_id: None,
                extraordinaria: None,
                pagada: Some(false),
                numero_quota: None,
            },
            Quota {
                user_id: "afiliado1".to_string(),
                monto: 250.0,
                fecha_vencimiento: Some("2025-02-01".to_string()),
                monto_pagado: 0.0,
                multa: 0.0,
                pagada_por: None,
                tipo: TipoQuota::Afiliado,
                loan_id: None,
                extraordinaria: None,
                pagada: Some(false),
                numero_quota: None,
            },
            Quota {
                user_id: "afiliado2".to_string(),
                monto: 250.0,
                fecha_vencimiento: Some("2025-01-01".to_string()),
                monto_pagado: 0.0,
                multa: 0.0,
                pagada_por: None,
                tipo: TipoQuota::Afiliado,
                loan_id: None,
                extraordinaria: None,
                pagada: Some(false),
                numero_quota: None,
            },
        ];
        for Quota in &quotas {
            let _ = context.quota_repo().save_quota(Quota.user_id.clone(), Quota);
        }
        // 4. Mock de get_all_users_for_affiliates si es necesario
        // 5. Ejecutar el resolver
        let result = futures::executor::block_on(
            QuotaQuery::get_monthly_affiliate_quota(&context, "TEST_ACCESS_TOKEN".to_string())
        ).unwrap();
        
        // Imprimir resultado para depuración
        println!("Resultado del resolver: {:?}", result);
        
        // 6. Validar formato y contenido del array de objetos QuotaAfiliadoMensualResponse
        assert!(!result.is_empty(), "El resultado no debe estar vacío");
        
        // Verificar que contiene quotas para Juan Perez (debería tener dos quotas: enero y febrero 2025)
        let quotas_juan: Vec<&QuotaAfiliadoMensualResponse> = result.iter()
            .filter(|r| r.identifier.contains("Juan Perez"))
            .collect();
        assert_eq!(quotas_juan.len(), 2, "Juan Perez debe tener exactamente 2 quotas");
        
        // Verificar campos específicos para las quotas de Juan Perez
        for Quota in &quotas_juan {
            assert_eq!(Quota.user_id, "TEST_ACCESS_TOKEN");
            assert_eq!(Quota.monto, 250.0);
            assert_eq!(Quota.nombre, "Juan Perez");
            assert_eq!(Quota.extraordinaria, false);
            // Verificar que el identifier tiene formato correcto con meses en español
            assert!(
                Quota.identifier == "Juan Perez - Enero 2025" || Quota.identifier == "Juan Perez - Febrero 2025",
                "Identifier incorrecto: {}", Quota.identifier
            );
            // Verificar fecha_vencimiento
            assert!(
                Quota.fecha_vencimiento == "2025-01-01" || Quota.fecha_vencimiento == "2025-02-01",
                "Fecha de vencimiento incorrecta: {}", Quota.fecha_vencimiento
            );
        }
        
        // Verificar que contiene quotas para Maria Gomez (debería tener una Quota: enero 2025)
        let quotas_maria: Vec<&QuotaAfiliadoMensualResponse> = result.iter()
            .filter(|r| r.identifier.contains("Maria Gomez"))
            .collect();
        assert_eq!(quotas_maria.len(), 1, "Maria Gomez debe tener exactamente 1 Quota");
        
        // Verificar campos específicos para la Quota de Maria Gomez
        let quota_maria = quotas_maria[0];
        assert_eq!(quota_maria.user_id, "TEST_ACCESS_TOKEN");
        assert_eq!(quota_maria.monto, 250.0);
        assert_eq!(quota_maria.nombre, "Maria Gomez");
        assert_eq!(quota_maria.extraordinaria, false);
        assert_eq!(quota_maria.identifier, "Maria Gomez - Enero 2025");
        assert_eq!(quota_maria.fecha_vencimiento, "2025-01-01");
        
        // Verificar que todas las quotas tienen fechas <= fecha actual (solo quotas de enero y febrero 2025)
        for Quota in &result {
            let fecha = NaiveDate::parse_from_str(&Quota.fecha_vencimiento, "%Y-%m-%d").unwrap();
            let hoy = chrono::Utc::now().date_naive();
            assert!(fecha <= hoy, "La Quota {} tiene fecha futura: {}", Quota.identifier, Quota.fecha_vencimiento);
        }
    }
}
