// Tests para la query de cuotas por id de usuario
// SCRUM-257: Validar que los datos coincidan con la tabla cuotas y los tipos sean correctos


use actix_web::web::Data;
use r2d2::Pool;
use redis::Client;
use general_api::models::graphql::{Cuota, TipoCuota};
use general_api::repos::graphql::cuota::CuotaRepo;

#[cfg(test)]
mod tests {
    #[test]
    fn test_filtrado_por_tipo_cuota() {
        let repo = get_test_repo();
        let access_token = "test_token_tipo";
        // Cuota tipo Prestamo
        let cuota_prestamo = Cuota {
            user_id: "user_2".to_string(),
            monto: 500.0,
            fecha_vencimiento: "2025-09-10".to_string(),
            monto_pagado: 0.0,
            multa: 0.0,
            pagada_por: None,
            tipo: TipoCuota::Prestamo,
            loan_id: Some("loan_xyz".to_string()),
            extraordinaria: None,
        };
        // Cuota tipo Afiliado
        let cuota_afiliado = Cuota {
            user_id: "user_2".to_string(),
            monto: 200.0,
            fecha_vencimiento: "2025-09-15".to_string(),
            monto_pagado: 0.0,
            multa: 0.0,
            pagada_por: None,
            tipo: TipoCuota::Afiliado,
            loan_id: None,
            extraordinaria: Some(false),
        };
        insert_cuota_test(&repo, access_token, &cuota_prestamo);
        insert_cuota_test(&repo, access_token, &cuota_afiliado);

        let result = repo.get_cuotas_pendientes(access_token.to_string());
        assert!(result.is_ok(), "La consulta de cuotas pendientes falló");
        let cuotas = result.unwrap();
        assert_eq!(cuotas.len(), 2, "Debe retornar dos cuotas pendientes");

        let mut found_prestamo = false;
        let mut found_afiliado = false;
        for cuota in cuotas {
            match &cuota.tipo {
                TipoCuota::Prestamo => {
                    assert_eq!(cuota.loan_id.as_deref(), Some("loan_xyz"));
                    found_prestamo = true;
                },
                TipoCuota::Afiliado => {
                    assert_eq!(cuota.extraordinaria, Some(false));
                    found_afiliado = true;
                },
            }
        }
        assert!(found_prestamo, "No se encontró cuota tipo Prestamo");
        assert!(found_afiliado, "No se encontró cuota tipo Afiliado");
    }
    use super::*;

    // Utilidad para crear un CuotaRepo de prueba
    fn get_test_repo() -> CuotaRepo {
        // Aquí deberías configurar un pool de Redis de prueba
        // Por ahora, se asume que existe un pool global para tests
        let pool = Data::new(Pool::new(Client::open("redis://127.0.0.1/").unwrap()).unwrap());
        CuotaRepo { pool }
    }

    // Utilidad para insertar cuotas de prueba
    fn insert_cuota_test(repo: &CuotaRepo, access_token: &str, cuota: &Cuota) {
        repo.save_cuota(access_token.to_string(), cuota).expect("No se pudo guardar la cuota de prueba");
    }

    #[test]
    fn test_consulta_basica_cuotas_pendientes() {
        let repo = get_test_repo();
        let access_token = "test_token_123";
        let cuota = Cuota {
            user_id: "user_1".to_string(),
            monto: 1000.0,
            fecha_vencimiento: "2025-09-01".to_string(),
            monto_pagado: 0.0,
            multa: 0.0,
            pagada_por: None,
            tipo: TipoCuota::Prestamo,
            loan_id: Some("loan_abc".to_string()),
            extraordinaria: None,
        };
        insert_cuota_test(&repo, access_token, &cuota);

        let result = repo.get_cuotas_pendientes(access_token.to_string());
        assert!(result.is_ok(), "La consulta de cuotas pendientes falló");
        let cuotas = result.unwrap();
        assert_eq!(cuotas.len(), 1, "Debe retornar una cuota pendiente");
        let returned = &cuotas[0];
        assert_eq!(returned.user_id, cuota.user_id);
        assert_eq!(returned.monto, cuota.monto);
        assert_eq!(returned.fecha_vencimiento, cuota.fecha_vencimiento);
        assert_eq!(returned.monto_pagado, cuota.monto_pagado);
        assert_eq!(returned.multa, cuota.multa);
        assert_eq!(returned.pagada_por, cuota.pagada_por);
        match &returned.tipo {
            TipoCuota::Prestamo => {
                assert_eq!(returned.loan_id.as_deref(), Some("loan_abc"));
            },
            _ => panic!("El tipo de cuota no es Prestamo"),
        }
    }
    // TODO: Implementar test de filtrado por tipo
    // TODO: Implementar test de consistencia de datos
    // TODO: Implementar test de casos límite
    // TODO: Implementar test de error y autenticación
}
