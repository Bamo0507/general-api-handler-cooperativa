# General Api Handler

Handler principal para todo el manejo del backend de la cooperativa, desde:

- Logeo
- Redis entries
- Auth

## Tecnologias en uso


## Pasos de Seteo

  - Escribir este comando para subir los containers

  ```bash
  docker-compose up --build
  ```

  o

  ```bash
  podman-compose up --build
  ```

## Requisitos para ejecutar los tests

Todos los tests de integración requieren que la variable de entorno `REDIS_URL` esté exportada en el CLI antes de ejecutar `cargo test`. No se utiliza `dotenv` ni valores por defecto; si la variable no está presente, los tests fallarán.

Ejemplo en PowerShell:

```powershell
$env:REDIS_URL="redis://127.0.0.1/"
> El compose file esta versionado de momento, solo para mostrar que si funciona el seteo
```

Ejemplo en bash:

```bash
export REDIS_URL="redis://127.0.0.1/"
> Luego se dejara de versionar
```

La base de datos Redis debe estar corriendo y accesible en la URL indicada.

> EL TESTING SE TIENE QUE HACER EN UN ENTORNO DE DESARROLLO, NO EN PRODUCCION
