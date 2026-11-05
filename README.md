# gc-proyecto-3

Proyecto 3 - Gráficas por Computadora

Software renderer escrito en Rust que muestra un sistema solar explorable con una nave. Todo el dibujo 3D ocurre en un framebuffer propio de 480x270 píxeles; raylib solo abre la ventana, lee el teclado y el mouse, mide el tiempo, muestra el framebuffer como textura y escribe el texto del HUD.

## Cómo compilar y ejecutar

```bash
cargo run --release
```

El juego carga los modelos desde `assets/models`. Si se ejecuta desde otra carpeta, busca la carpeta junto al ejecutable y en la raíz del proyecto.

Pruebas:

```bash
cargo test
```

## Controles

| Tecla | Acción |
| --- | --- |
| `W` / `S` | Acelerar / frenar y retroceder |
| `A` / `D` o flechas izquierda / derecha | Girar (yaw) |
| Flechas arriba / abajo | Subir o bajar la nariz (pitch) |
| Clic derecho + mouse | Apuntar la nave con el mouse |
| `Q` / `E` | Roll |
| `Espacio` / `Ctrl` | Subir / bajar |
| `Shift` | Turbo |
| `C` | Cambiar entre cámara de seguimiento y cámara libre |
| Rueda del mouse | Distancia de la cámara |
| `O` | Entrar o salir del modo órbita alrededor del planeta cercano |
| `1` a `5` | Elegir planeta destino |
| `Tab` | Siguiente destino (incluye el Sol y la Luna) |
| `F` | Warp al destino |
| `-` / `=` | Velocidad de las órbitas en vivo |
| `M` | Métricas de rendimiento |
| `H` | Panel de controles |
| `Esc` | Cerrar el panel o salir |

En modo órbita, `W` / `S` cambian la velocidad, `Espacio` / `Ctrl` la altura y `A` / `D` el rumbo alrededor del planeta.

## Modelos

| Archivo | Uso |
| --- | --- |
| `ETA_2.obj` | Nave del jugador |
| `ENEMY1.obj` | Naves que patrullan alrededor de los planetas |
| `Planet.obj` | Sol, planeta rocoso y planeta gaseoso |
| `CORE_.obj` | Planeta núcleo |
| `Droid_confed.obj` | Estación droide |

Los cristales de la superficie se generan por código. Si se agrega `assets/models/crystal.obj`, el juego lo usa en su lugar sin cambiar código. Si falta cualquier modelo, el juego usa una malla temporal y avisa en la consola.

## Características de la rúbrica y dónde se ven

| Característica | Dónde se ve | Código |
| --- | --- | --- |
| Sistema solar con 7 cuerpos (Sol, Vulcano, Núcleo, Rocoso, Gaseoso, Droide y Luna) | Vista inicial | `src/world/solar_system.rs` |
| Órbitas en el tiempo, con inclinación, excentricidad y rotación propia; luna que orbita un planeta | Órbitas punteadas violeta; la Luna gira alrededor de Gaseoso | `src/world/orbit.rs` |
| Cinturón de asteroides procedural con semilla fija | Entre Rocoso y Gaseoso | `src/world/generation.rs` |
| Objetos sobre la superficie orientados por la normal y que giran con el planeta | Cristales y rocas en Rocoso y en la Luna | `src/world/generation.rs` |
| Nave que sigue a la cámara | Nave abajo al centro, vista desde atrás y arriba | `src/game/mod.rs` |
| Movimiento 3D con inercia | `W`, `S`, `A`, `D`, flechas, `Q`, `E`, `Espacio`, `Ctrl`, `Shift` | `src/game/player.rs` |
| Modo órbita local | `O` cerca de un planeta | `src/game/player.rs` |
| Colisiones de nave y cámara contra el Sol, planetas, luna y asteroides | Intentar atravesar un cuerpo | `src/engine/collision.rs` |
| Warp animado hacia un destino que se mueve | `1` a `5` o `Tab`, luego `F` | `src/game/warp.rs` |
| Acercamiento y alejamiento sin pantallas de carga | Volar hacia cualquier planeta | Todo ocurre en la misma escena |
| HUD, mira y panel de controles | Barra inferior, `+` al centro, `H` | `src/game/hud.rs` |
| Paso fijo de simulación de 1/60 s | Todo el movimiento | `src/app.rs`, `src/platform/time.rs` |
| Cargador OBJ propio y mallas compartidas | Todos los modelos | `src/assets/` |

## Recorrido sugerido para la demostración

1. Al iniciar, la nave mira hacia el Sol desde fuera del sistema.
2. Pulsar `3` y luego `F` para hacer warp hacia Rocoso.
3. Acercarse con `W` hasta ver los cristales de la superficie.
4. Pulsar `O` para rodear el planeta y otra vez `O` para volver al vuelo libre.
5. Alejarse con `S` o con turbo y elegir otro destino con `Tab`.

## Integrantes

- Pendiente
- Pendiente

## Contribuciones

- Pendiente

## Uso de herramientas de apoyo

- Pendiente
