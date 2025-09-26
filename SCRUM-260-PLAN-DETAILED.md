---

## TODO's actuales para SCRUM-260

- [ ] Analizar estructura para mutación
  - Antes de diseñar la mutación, analizar la definición actual de PaymentMutation en src/endpoints/handlers/graphql/payment.rs, revisar mutaciones existentes, verificar el modelo Payment en src/models/graphql.rs y documentar convenciones observadas.
- [ ] Diseñar mutación GraphQL
  - Definir la mutación en PaymentMutation, especificando los parámetros requeridos (id, state, commentary) y el tipo de retorno. Fundamentar el diseño en los patrones actuales del proyecto.
- [ ] Analizar estructura de PaymentRepo
  - Antes de implementar el método, analizar la estructura de PaymentRepo en src/repos/graphql/payment.rs, revisar métodos existentes para acceso y persistencia en Redis, confirmar uso de helpers y patrones, y documentar el manejo de errores.
- [ ] Implementar método en PaymentRepo
  - Agregar el método en src/repos/graphql/payment.rs que actualice los campos state y commentary de un pago en Redis, siguiendo el patrón de acceso y persistencia actual.
- [ ] Analizar registro en schema
  - Antes de registrar la mutación, revisar cómo se registran las mutaciones en el schema principal, analizar el endpoint /graphql/payment y validar el proceso de registro.
- [ ] Registrar mutación en schema
  - Asegurarse de que la mutación esté expuesta correctamente en el schema y disponible en el endpoint /graphql/payment.
- [ ] Analizar estructura de pruebas
  - Antes de crear pruebas, analizar los tests existentes en tests/graphql/payment_test.rs, revisar helpers y asserts, y documentar el patrón de pruebas.
- [ ] Crear pruebas unitarias
  - Agregar tests en tests/graphql/payment_test.rs que inserten un pago, ejecuten la mutación y validen que los cambios se reflejan correctamente en la persistencia.
- [ ] Analizar documentación existente
  - Antes de documentar, revisar la documentación y comentarios existentes en los archivos relevantes, y confirmar que cada decisión esté fundamentada.
- [ ] Documentar fundamentos y decisiones
  - Agregar comentarios y documentación mínima en el código y en el plan para justificar cada decisión y asegurar trazabilidad.
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

  1. Agregar comentarios en el código justificando cada decisión y referencia a este plan.
  2. Documentar la mutación en el archivo de instrucciones/copilot si aplica.
  3. Incluir referencias a los patrones y convenciones seguidos.
  - Garantizar que cualquier desarrollador/IA pueda entender el propósito, dependencias y criterios de aceptación sin ambigüedad.

## Análisis y convenciones para la mutación de actualización de pago

### Propósito de la mutación
La mutación permitirá actualizar el estado (`state`) y el comentario (`commentary`) de un pago existente en el sistema. Es clave para la gestión administrativa, permitiendo aprobar, rechazar o dejar observaciones sobre el pago, y guardar los cambios en la persistencia (Redis).

### Estructura actual
- En `src/endpoints/handlers/graphql/payment.rs` solo existen queries (`PaymentQuery`), no hay mutaciones implementadas para pagos.
- El patrón de queries/mutaciones es: método público, async, recibe contexto y parámetros, retorna `Result<T, String>`.
- El acceso a datos se delega al repo vía `context.payment_repo()`.

### Modelo Payment
- Ubicación: `src/models/graphql.rs`

### Convención recomendada para el estado
Se recomienda usar el enum:

```rust
#[derive(Clone, Serialize, Deserialize, Debug, GraphQLEnum, PartialEq)]
pub enum PaymentStatus {
  OnRevision,
  Rejected,
  Accepted,
  ParsedError,
}

impl PaymentStatus {
  pub fn from_string(raw_status: String) -> PaymentStatus {
    match raw_status.to_uppercase().as_str() {
      "ON_REVISION" => PaymentStatus::OnRevision,
      "REJECTED" => PaymentStatus::Rejected,
      "ACCEPTED" => PaymentStatus::Accepted,
      _ => PaymentStatus::ParsedError,
    }
  }
}
```

Ventajas:
- El campo `state` de `Payment` puede ser de tipo `PaymentStatus` en vez de `String`, forzando valores válidos.
- El derive `GraphQLEnum` permite exponer el enum directamente en el schema GraphQL.
- El método `from_string` ayuda a convertir strings externos al enum, manejando errores.

### Recomendaciones para la implementación
- Actualizar el modelo `Payment` para que el campo `state` sea de tipo `PaymentStatus`.
- Usar el enum en la mutación para recibir y devolver el estado del pago.
- Mantener el patrón de manejo de errores y wrappers del proyecto.

### Trazabilidad
Toda esta información debe ser la base para el desarrollo en cualquier branch y chat futuro.

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
