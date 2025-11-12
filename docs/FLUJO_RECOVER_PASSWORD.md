# Flujo de Endpoints - Recuperación de Contraseña

## Prerrequisito: Crear Usuario y Configurar Respuestas de Seguridad

### 1. Sign Up
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

Guardar el `access_token` retornado.

---

### 2. Obtener Preguntas de Seguridad
**GET** `/general/security-questions?user_name={user_name}`

**Respuesta:**
```json
{
  "questions": [
    "¿Cuál fue el nombre de la primera escuela o colegio al que asististe?", //index 0
    "¿En qué colonia o barrio viviste durante tu infancia?", //index 1
    "¿Cuál era tu materia o clase favorita en la escuela?" //index 2    
  ]
}
```

---

### 3. Configurar Respuestas de Seguridad
**POST** `/general/configure-security-answers`

Requiere el `access_token` del paso 1.

```json
{
  "access_token": "string_del_signup",
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

### 4. Resetear Contraseña
**POST** `/general/reset-password`

```json
{
  "user_name": "string",
  "question_index": 0,
  "security_answer": "string",
  "new_pass_code": "string"
}
```

**Respuesta:**
```json
{
  "user_name": "string",
  "access_token": "string_nuevo",
  "user_type": "General"
}
```

El nuevo `access_token` es válido con la nueva contraseña. El token anterior se invalida automáticamente.

---

## Notas
- Las respuestas de seguridad se guardan hasheadas y normalizadas (minúsculas sin espacios al inicio/final).
- Cada usuario tiene 3 preguntas de seguridad fijas.
- Al resetear contraseña, todos los datos del usuario se preservan excepto las credenciales.
