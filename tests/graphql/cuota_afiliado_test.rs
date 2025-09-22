use general_api::endpoints::handlers::configs::schema::GeneralContext;
use general_api::models::graphql::{Cuota, TipoCuota};
use general_api::endpoints::handlers::graphql::cuota::CuotaQuery;
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
        // 6. Validar formato y contenido (meses en español y solo hasta la fecha actual)
        let esperado_juan = [
            "juan perez - enero 2025",
            "juan perez - febrero 2025"
        ];
        let esperado_maria = [
            "maria gomez - enero 2025"
        ];
        let result_lower: Vec<String> = result.iter().map(|s| s.to_lowercase()).collect();
        assert!(esperado_juan.iter().any(|e| result_lower.contains(&e.to_string())));
        assert!(esperado_maria.iter().any(|e| result_lower.contains(&e.to_string())));
    }
}
