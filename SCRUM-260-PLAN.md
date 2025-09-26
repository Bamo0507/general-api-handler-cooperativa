# SCRUM-260: Planificación detallada para la mutación de actualización de pagos

## Objetivo
Implementar una mutación GraphQL que permita cambiar el estado y el comentario de un pago, validando que ambos se actualizan correctamente en la persistencia.

---

## 1. Diseñar mutación GraphQL para actualizar estado y comentario de pago
- Definir la mutación en `PaymentMutation` (`src/endpoints/handlers/graphql/payment.rs`).
- Parámetros: `id` (String), `state` (String), `commentary` (String).
- Retorno: `Result<Payment, String>` para devolver el pago actualizado.
- Basado en el patrón de mutaciones y queries existentes.

## 2. Implementar método en PaymentRepo para actualizar pago
- Agregar método en `src/repos/graphql/payment.rs` que reciba los parámetros y actualice los campos en Redis.
- Usar el patrón de acceso y persistencia actual (pool, helpers, manejo de errores).
- Validar que solo se actualicen los campos requeridos.

## 3. Registrar mutación en el schema GraphQL
- Asegurarse de que la mutación esté expuesta en el schema y disponible en `/graphql/payment`.
- Seguir el patrón de registro de mutaciones en el proyecto.

## 4. Crear pruebas unitarias para la mutación
- En `tests/graphql/payment_test.rs`, insertar un pago de prueba.
- Ejecutar la mutación para cambiar estado y comentario.
- Validar que los cambios se reflejan correctamente en la persistencia y en la respuesta.

## 5. Documentar fundamentos y decisiones en el código
- Agregar comentarios y documentación mínima en el código y en el plan.
- Justificar cada decisión y asegurar trazabilidad.

---

## Trazabilidad y fundamentos
- Cada paso está fundamentado en los patrones y archivos actuales del proyecto.
- No se asume ningún método, campo ni estructura adicional fuera de lo que existe en el código.
- La implementación debe seguir la modularidad, nomenclatura y buenas prácticas ya establecidas.
