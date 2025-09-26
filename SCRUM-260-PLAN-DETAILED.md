# SCRUM-260: Planeación exhaustiva para la mutación de actualización de pagos

## Objetivo

Implementar una mutación GraphQL que permita cambiar el estado y el comentario de un pago, validando que ambos se actualizan correctamente en la persistencia (Redis) y que la API cumple con los patrones, convenciones y trazabilidad del proyecto.

---

## 1. Análisis de contexto y dependencias

- **Modelo de pago:**
  - Ubicación: `src/models/graphql.rs`, struct `Payment`.
  - Campos relevantes: `id`, `state`, `commentary` (verificar nombres exactos en el modelo).
  - El modelo debe estar alineado con el DDL y reflejar los campos persistidos en Redis.
- **Repositorio de pagos:**
  - Ubicación: `src/repos/graphql/payment.rs`, struct `PaymentRepo`.
  - Métodos existentes: `get_all_payments`, `get_user_payments`.
  - Se requiere agregar un método para actualizar los campos `state` y `commentary` de un pago existente.
- **Mutaciones GraphQL:**
  - Ubicación: `src/endpoints/handlers/graphql/payment.rs`, struct `PaymentMutation`.
  - Seguir el patrón de definición de mutaciones existente (ver otras mutaciones en el proyecto).
- **Pruebas unitarias:**
  - Ubicación: `tests/graphql/payment_test.rs`.
  - Usar helpers y patrones de limpieza de Redis ya implementados.
- **Convenciones y documentación:**
  - Revisar y cumplir con lo documentado en `copilot-instructions.md` (wrappers, DDL friendly, GraphQL friendly, idioma, buenas prácticas).

---

## 2. Diseño de la mutación GraphQL

- **Nombre sugerido:** `updatePaymentStateAndCommentary`
- **Ubicación:** `PaymentMutation` en `src/endpoints/handlers/graphql/payment.rs`
- **Parámetros requeridos:**
  - `id: String!` (identificador único del pago)
  - `state: String!` (nuevo estado)
  - `commentary: String!` (nuevo comentario)
- **Tipo de retorno:**
  - `Payment` actualizado, o error descriptivo (`Result<Payment, String>` en Rust)
- **Fundamento:**
  - El diseño debe seguir el patrón de mutaciones existentes, usando wrappers y manejo de errores consistente.
  - No se deben asumir campos adicionales ni lógica fuera de lo que existe en el modelo y repositorio.

---

## 3. Implementación del método en PaymentRepo

- **Ubicación:** `src/repos/graphql/payment.rs`
- **Nombre sugerido:** `update_payment_state_and_commentary`
- **Firma:**
  - Recibe: `id: &str`, `state: &str`, `commentary: &str`, `redis_pool: &r2d2::Pool<redis::Client>`
  - Retorna: `Result<Payment, String>`
- **Lógica:**
  1. Buscar el pago por `id` en Redis.
  2. Si no existe, retornar error descriptivo.
  3. Actualizar solo los campos `state` y `commentary`.
  4. Persistir el pago actualizado en Redis.
  5. Retornar el pago actualizado.
- **Fundamento:**
  - Usar helpers y patrones de acceso/persistencia ya implementados.
  - Manejar errores y validaciones siguiendo el estándar del proyecto.

---

## 4. Registro de la mutación en el schema GraphQL

- **Ubicación:** `src/endpoints/handlers/graphql/payment.rs` y archivos de registro de schema.
- **Acciones:**
  1. Agregar la mutación a la struct `PaymentMutation`.
  2. Registrar la mutación en el schema principal para que esté disponible en `/graphql/payment`.
  3. Validar que la mutación aparece en el introspection query de GraphQL.
- **Fundamento:**
  - Seguir el patrón de registro de mutaciones y queries existente.
  - No modificar el endpoint ni la estructura del schema fuera de lo necesario.

---

## 5. Pruebas unitarias exhaustivas

- **Ubicación:** `tests/graphql/payment_test.rs`
- **Acciones:**
  1. Insertar un pago de prueba en Redis usando los helpers existentes.
  2. Ejecutar la mutación con parámetros nuevos para `state` y `commentary`.
  3. Validar que la respuesta contiene el pago actualizado.
  4. Consultar el pago en Redis y verificar que los campos fueron actualizados correctamente.
  5. Probar casos de error: pago inexistente, parámetros inválidos, persistencia fallida.
- **Fundamento:**
  - Usar helpers y patrones de limpieza de Redis para evitar efectos colaterales.
  - Seguir la convención de asserts y manejo de errores en los tests existentes.

---

## 6. Documentación y trazabilidad

- **Acciones:**
  1. Agregar comentarios en el código justificando cada decisión y referencia a este plan.
  2. Documentar la mutación en el archivo de instrucciones/copilot si aplica.
  3. Incluir referencias a los patrones y convenciones seguidos.
- **Fundamento:**
  - Garantizar que cualquier desarrollador/IA pueda entender el propósito, dependencias y criterios de aceptación sin ambigüedad.

---

## 7. Criterios de aceptación

- La mutación actualiza correctamente los campos `state` y `commentary` de un pago existente en Redis.
- La API responde con el pago actualizado o error descriptivo según corresponda.
- El código cumple con las convenciones de wrappers, DDL friendly, GraphQL friendly, idioma y buenas prácticas documentadas.
- Las pruebas unitarias cubren casos exitosos y de error, validando persistencia y respuesta.
- Toda la lógica, dependencias y decisiones están documentadas y justificadas en el código y en este plan.

---

## 8. Referencias y convenciones

- **Archivos clave:**
  - Modelo: `src/models/graphql.rs`
  - Repositorio: `src/repos/graphql/payment.rs`
  - Mutación: `src/endpoints/handlers/graphql/payment.rs`
  - Pruebas: `tests/graphql/payment_test.rs`
  - Convenciones: `copilot-instructions.md`
- **Patrones:**
  - Uso de wrappers para respuestas y errores.
  - Persistencia en Redis usando pool y helpers.
  - Registro de mutaciones en el schema siguiendo el patrón del proyecto.
  - Pruebas con helpers y limpieza de Redis.
  - Documentación mínima y trazabilidad en cada paso.

---

## 9. Notas finales

- No se debe asumir ningún campo, método ni lógica fuera de lo que existe en el modelo, repositorio y patrones del proyecto.
- Cada paso debe ser trazable y justificable con referencia a este plan y a los archivos/documentación del proyecto.
- Si surge alguna ambigüedad, se debe documentar y fundamentar la decisión tomada.

---

**Fin de la planeación exhaustiva para SCRUM-260.**
