# task_manager_sql

API CRUD básica de tareas con **Rust + Axum + SQLx + SQLite**.

## Requisitos

- Rust estable (instalable con [rustup](https://rustup.rs/))
- SQLite (opcional para inspección manual de la BD)

## Configuración

1. Clona el repositorio.
2. Crea tu archivo de entorno local:

```bash
cp .env.example .env
```

3. Ajusta `DATABASE_URL` si quieres usar otro nombre/ruta de base de datos.

## Ejecutar la aplicación

```bash
cargo run
```

La app arranca en `http://127.0.0.1:3000` y ejecuta automáticamente las migraciones de `./migrations` al iniciar.

## Migraciones

Este proyecto usa `sqlx::migrate!("./migrations")` en el arranque, por lo que no hace falta un paso manual adicional para aplicar `0001_tasks.sql`.

## Endpoints CRUD disponibles

Rutas definidas en el código:

- `GET /task` → lista tareas
- `POST /task` → crea tarea
- `PATCH /task/{id}` → actualiza campos de una tarea
- `DELETE /task/{id}` → elimina tarea

## Ejemplos de uso

### Crear tarea

```bash
curl -X POST http://127.0.0.1:3000/task \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Estudiar Axum",
    "description": "Repasar handlers y extractors",
    "status": "pending",
    "priority": 2
  }'
```

### Listar tareas

```bash
curl http://127.0.0.1:3000/task
```

### Actualizar tarea

```bash
curl -X PATCH http://127.0.0.1:3000/task/1 \
  -H "Content-Type: application/json" \
  -d '{
    "status": "completed",
    "priority": 3
  }'
```

### Eliminar tarea

```bash
curl -X DELETE http://127.0.0.1:3000/task/1
```

## Comprobaciones de calidad

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features
```
