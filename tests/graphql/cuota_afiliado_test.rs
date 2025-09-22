use general_api::endpoints::handlers::configs::schema::GeneralContext;
use general_api::models::graphql::{Cuota, TipoCuota};
use general_api::endpoints::handlers::graphql::cuota::{CuotaQuery, CuotaAfiliadoMensualResponse};
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
    fn test_get_cuotas_afiliado_mensuales_formateadas() {
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

        // 2. Crear cuotas pendientes para cada afiliado y guardar en Redis
        let cuotas = vec![
            // Para cada afiliado, crear una cuota en dos meses distintos
            Cuota {
                user_id: "afiliado1".to_string(),
                monto: 250.0,
                fecha_vencimiento: Some("2025-01-01".to_string()),
                monto_pagado: 0.0,
                multa: 0.0,
                pagada_por: None,
                tipo: TipoCuota::Afiliado,
                loan_id: None,
                extraordinaria: None,
                pagada: Some(false),
                numero_cuota: None,
            },
            Cuota {
                user_id: "afiliado1".to_string(),
                monto: 250.0,
                fecha_vencimiento: Some("2025-02-01".to_string()),
                monto_pagado: 0.0,
                multa: 0.0,
                pagada_por: None,
                tipo: TipoCuota::Afiliado,
                loan_id: None,
                extraordinaria: None,
                pagada: Some(false),
                numero_cuota: None,
            },
            Cuota {
                user_id: "afiliado2".to_string(),
                monto: 250.0,
                fecha_vencimiento: Some("2025-01-01".to_string()),
                monto_pagado: 0.0,
                multa: 0.0,
                pagada_por: None,
                tipo: TipoCuota::Afiliado,
                loan_id: None,
                extraordinaria: None,
                pagada: Some(false),
                numero_cuota: None,
            },
        ];
        for cuota in &cuotas {
            let _ = context.cuota_repo().save_cuota(cuota.user_id.clone(), cuota);
        }
        // 4. Mock de get_all_users_for_affiliates si es necesario
        // 5. Ejecutar el resolver
        let result = futures::executor::block_on(
            CuotaQuery::get_cuotas_afiliado_mensuales_formateadas(&context)
        ).unwrap();
        
        // Imprimir resultado para depuración
        println!("Resultado del resolver: {:?}", result);
        
        // 6. Validar formato y contenido del array de objetos CuotaAfiliadoMensualResponse
        assert!(!result.is_empty(), "El resultado no debe estar vacío");
        
        // Verificar que contiene cuotas para Juan Perez (debería tener dos cuotas: enero y febrero 2025)
        let cuotas_juan: Vec<&CuotaAfiliadoMensualResponse> = result.iter()
            .filter(|r| r.user_id == "afiliado1")
            .collect();
        assert_eq!(cuotas_juan.len(), 2, "Juan Perez debe tener exactamente 2 cuotas");
        
        // Verificar campos específicos para las cuotas de Juan Perez
        for cuota in &cuotas_juan {
            assert_eq!(cuota.user_id, "afiliado1");
            assert_eq!(cuota.monto, 250.0);
            assert_eq!(cuota.nombre, "Juan Perez");
            assert_eq!(cuota.extraordinaria, false);
            // Verificar que el identifier tiene formato correcto con meses en español
            assert!(
                cuota.identifier == "Juan Perez - Enero 2025" || cuota.identifier == "Juan Perez - Febrero 2025",
                "Identifier incorrecto: {}", cuota.identifier
            );
            // Verificar fecha_vencimiento
            assert!(
                cuota.fecha_vencimiento == "2025-01-01" || cuota.fecha_vencimiento == "2025-02-01",
                "Fecha de vencimiento incorrecta: {}", cuota.fecha_vencimiento
            );
        }
        
        // Verificar que contiene cuotas para Maria Gomez (debería tener una cuota: enero 2025)
        let cuotas_maria: Vec<&CuotaAfiliadoMensualResponse> = result.iter()
            .filter(|r| r.user_id == "afiliado2")
            .collect();
        assert_eq!(cuotas_maria.len(), 1, "Maria Gomez debe tener exactamente 1 cuota");
        
        // Verificar campos específicos para la cuota de Maria Gomez
        let cuota_maria = cuotas_maria[0];
        assert_eq!(cuota_maria.user_id, "afiliado2");
        assert_eq!(cuota_maria.monto, 250.0);
        assert_eq!(cuota_maria.nombre, "Maria Gomez");
        assert_eq!(cuota_maria.extraordinaria, false);
        assert_eq!(cuota_maria.identifier, "Maria Gomez - Enero 2025");
        assert_eq!(cuota_maria.fecha_vencimiento, "2025-01-01");
        
        // Verificar que todas las cuotas tienen fechas <= fecha actual (solo cuotas de enero y febrero 2025)
        for cuota in &result {
            let fecha = NaiveDate::parse_from_str(&cuota.fecha_vencimiento, "%Y-%m-%d").unwrap();
            let hoy = chrono::Utc::now().date_naive();
            assert!(fecha <= hoy, "La cuota {} tiene fecha futura: {}", cuota.identifier, cuota.fecha_vencimiento);
        }
    }
}
