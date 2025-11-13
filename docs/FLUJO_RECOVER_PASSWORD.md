# Flujo de Endpoints - Recuperación de Contraseña

## Prerrequisito: Crear Usuario y Configurar Respuestas de Seguridad

### 1. Sign Up (Crear Usuario)
**POST** `/general/signup`

```json
{
  "user_name": "string",
  "pass_code": "string",
  "real_name": "string"
}
```

**Respuesta:**
```json
{
  "user_name": "string",
  "access_token": "string",
  "user_type": "General"
}
```

Guardar el `access_token` retornado para usar en los próximos pasos.

---

### 2. Login (Obtener Access Token)
**GET** `/general/login?user_name={user_name}&pass_code={pass_code}`

Permite login con credenciales existentes si ya el usuario se registró.

**Respuesta:**
```json
{
  "user_name": "string",
  "access_token": "string",
  "user_type": "General"
}
```

---

### 3. Obtener Preguntas de Seguridad
**GET** `/general/security-questions?user_name={user_name}`

Retorna las 3 preguntas de seguridad disponibles para el usuario.

**Respuesta:**
```json
{
  "questions": [
    "¿Cuál fue el nombre de la primera escuela o colegio al que asististe?", // index 0
    "¿Cuál era el nombre de tu mejor amigo de la infancia",                  // index 1
    "¿Cuál era tu materia o clase favorita en la escuela?"                     // index 2    
  ]
}
```

---

### 4. Configurar Respuestas de Seguridad
**POST** `/general/configure-security-answers`

Requiere el `access_token` del paso 1 o 2.

```json
{
  "access_token": "string_del_signup_o_login",
  "answers": [
    "respuesta_pregunta_0",
    "respuesta_pregunta_1",
    "respuesta_pregunta_2"
  ]
}
```

**Respuesta:**
```json
{
  "message": "Respuestas de seguridad guardadas correctamente"
}
```

---

## Flujo de Recuperación de Contraseña

### 5. [OPCIONAL] Validar Respuesta de Seguridad
**POST** `/general/validate-security-answer`

Valida si una respuesta de seguridad es correcta **sin resetear la contraseña**. Útil para verificar respuestas antes de hacer el reset.

```json
{
  "user_name": "string",
  "question_index": 0,
  "security_answer": "string"
}
```

**Respuesta (correcta):**
```json
{
  "message": "Respuesta válida"
}
```

**Respuesta (incorrecta):**
```json
{
  "message": "Respuesta incorrecta"
}
```

---

### 6. Resetear Contraseña
**POST** `/general/reset-password`

Valida la respuesta de seguridad y resetea la contraseña, generando un nuevo `access_token`.

```json
{
  "user_name": "string",
  "question_index": 0,
  "security_answer": "string",
  "new_pass_code": "string"
}
```

**Respuesta (exitosa):**
```json
{
  "user_name": "string",
  "access_token": "string_nuevo",
  "user_type": "General"
}
```

**Respuesta (error):**
```json
{
  "message": "string de error"
}
```

El nuevo `access_token` es válido con la nueva contraseña. El token anterior se invalida automáticamente.

---

## Mapeo de Datos en Reset de Contraseña

Cuando un usuario resetea su contraseña, se genera un nuevo `db_composite_key` (derivado del nuevo `access_token`). **TODOS** los siguientes datos se copian automáticamente:


1. **Datos Base:**
   - Nombre completo (`complete_name`)
   - Clave de afiliado (`affiliate_key`)

2. **Datos Financieros:**
   - Dinero pagado al capital (`payed_to_capital`)
   - Capital adeudado (`owed_capital`)

3. **Datos de Rol:**
   - Flag de directivo (`is_directive`)

4. **Respuestas de Seguridad:**
   - Las 3 respuestas de seguridad hasheadas (indices 0, 1, 2)

5. **Datos de Préstamos, Pagos y Multas (JSON):**
   - **Payments**: Todos los pagos del usuario (`users:{db_key}:payments:*`)
   - **Loans**: Todos los préstamos del usuario (`users:{db_key}:loans:*`)
   - **Fines**: Todas las multas del usuario (`users:{db_key}:fines:*`)

### Proceso de Mapeo

1. Se obtiene el `db_composite_key` anterior (del token actual)
2. Se genera el nuevo `db_composite_key` (del nuevo token)
3. Se copian **todos** los datos listados arriba a las nuevas claves
4. Se elimina **completamente** todas las claves antiguas (seguridad: invalida el token anterior)
5. Se actualiza el mapeo `affiliate_key_to_db_access` al nuevo `db_composite_key`


## Notas

- Las respuestas de seguridad se guardan hasheadas y normalizadas (minúsculas sin espacios al inicio/final).
- Cada usuario tiene 3 preguntas de seguridad fijas (índices 0, 1, 2).
- Al resetear contraseña, **todos los datos del usuario se preservan** (loans, payments, fines, dinero adeudado, etc.) excepto el `access_token` que cambia.
- El flujo típico es: Sign Up → Obtener Preguntas → Configurar Respuestas → (luego si olvida contraseña) → Resetear Contraseña
- El endpoint `validate-security-answer` es opcional y se usa solo si quieres validar una respuesta sin hacer el reset.
