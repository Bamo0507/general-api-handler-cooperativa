// Tests para la query de quotas por id de usuario
// SCRUM-257: Validar que los datos coincidan con la tabla quotas y los tipos sean correctos


use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;
use general_api::models::graphql::{Quota, TipoQuota};
use general_api::repos::graphql::Quota::QuotaRepo;

#[cfg(test)]
mod tests {
    #[test]
    fn test_filtrado_por_tipo_quota() {
        let repo = get_test_repo();
        let access_token = "test_token_tipo";
        // Quota tipo Prestamo
        let quota_prestamo = Quota {
            user_id: "user_2".to_string(),
            monto: 500.0,
            fecha_vencimiento: Some("2025-09-10".to_string()),
            monto_pagado: 0.0,
            multa: 0.0,
            pagada_por: None,
            tipo: TipoQuota::Prestamo,
            loan_id: Some("loan_xyz".to_string()),
            extraordinaria: None,
            pagada: None,
            numero_quota: Some(1),
        };
        // Quota tipo Afiliado
        let quota_afiliado = Quota {
            user_id: "user_2".to_string(),
            monto: 200.0,
            fecha_vencimiento: Some("2025-09-15".to_string()),
            monto_pagado: 0.0,
            multa: 0.0,
            pagada_por: None,
            tipo: TipoQuota::Afiliado,
            loan_id: None,
            extraordinaria: Some(false),
            pagada: None,
            numero_quota: None,
        };
        insert_quota_test(&repo, access_token, &quota_prestamo).expect("No se pudo guardar la Quota de prueba (Prestamo)");
        insert_quota_test(&repo, access_token, &quota_afiliado).expect("No se pudo guardar la Quota de prueba (Afiliado)");

        let result = repo.get_pending_quotas(access_token.to_string());
        assert!(result.is_ok(), "La consulta de quotas pendientes falló");
        let quotas = result.unwrap();
        assert_eq!(quotas.len(), 2, "Debe retornar dos quotas pendientes");

        let mut found_prestamo = false;
        let mut found_afiliado = false;
        for Quota in quotas {
            match &Quota.tipo {
                TipoQuota::Prestamo => {
                    assert_eq!(Quota.loan_id.as_deref(), Some("loan_xyz"));
                    assert_eq!(Quota.numero_quota, Some(1));
                    found_prestamo = true;
                },
                TipoQuota::Afiliado => {
                    assert_eq!(Quota.extraordinaria, Some(false));
                    assert_eq!(Quota.numero_quota, None);
                    found_afiliado = true;
                },
            }
        }
        assert!(found_prestamo, "No se encontró Quota tipo Prestamo");
        assert!(found_afiliado, "No se encontró Quota tipo Afiliado");
    }
    use super::*;

    use general_api::repos::graphql::utils::create_test_context;

    // Utilidad para crear un QuotaRepo de prueba usando el helper central del proyecto
    fn get_test_repo() -> QuotaRepo {
        // Usa una variable de entorno para la URL de Redis en vez de hardcodear
        let redis_url = std::env::var("REDIS_TEST_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let pool = Data::new(Pool::new(Client::open(redis_url).unwrap()).unwrap());
        QuotaRepo { pool }
    }

    // Utilidad para insertar quotas de prueba
    fn insert_quota_test(repo: &QuotaRepo, access_token: &str, Quota: &Quota) -> Result<(), Box<dyn std::error::Error>> {
        repo.save_quota(access_token.to_string(), Quota)?;
        Ok(())
    }

    #[test]
    fn test_consulta_basica_quotas_pendientes() {
        let repo = get_test_repo();
        let access_token = "test_token_123";
        let Quota = Quota {
            user_id: "user_1".to_string(),
            monto: 1000.0,
            fecha_vencimiento: "2025-09-01".to_string(),
            monto_pagado: 0.0,
            multa: 0.0,
            pagada_por: None,
            tipo: TipoQuota::Prestamo,
            loan_id: Some("loan_abc".to_string()),
            extraordinaria: None,
        };
        insert_quota_test(&repo, access_token, &Quota).expect("No se pudo guardar la Quota de prueba");

        let result = repo.get_pending_quotas(access_token.to_string());
        assert!(result.is_ok(), "La consulta de quotas pendientes falló");
        let quotas = result.unwrap();
        assert_eq!(quotas.len(), 1, "Debe retornar una Quota pendiente");
        let returned = &quotas[0];
        assert_eq!(returned.user_id, Quota.user_id);
        assert_eq!(returned.monto, Quota.monto);
        assert_eq!(returned.fecha_vencimiento, Quota.fecha_vencimiento);
        assert_eq!(returned.monto_pagado, Quota.monto_pagado);
        assert_eq!(returned.multa, Quota.multa);
        assert_eq!(returned.pagada_por, Quota.pagada_por);
        match &returned.tipo {
            TipoQuota::Prestamo => {
                assert_eq!(returned.loan_id.as_deref(), Some("loan_abc"));
            },
            _ => panic!("El tipo de Quota no es Prestamo"),
        }
    }
    // TODO: Implementar test de filtrado por tipo
    // TODO: Implementar test de consistencia de datos
    // TODO: Implementar test de casos límite
    // TODO: Implementar test de error y autenticación
}
