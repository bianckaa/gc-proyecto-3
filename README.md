# gc-proyecto-3

Proyecto 3 - Gráficas por Computadora

Software renderer escrito en Rust que muestra un sistema solar explorable con una nave. El programa inicia en un menú 3D de gran escala y entra al juego al volar dentro del Sol. Todo el dibujo 3D ocurre en un framebuffer propio de 480x270 píxeles; raylib solo abre la ventana, lee el teclado y el mouse, mide el tiempo, muestra el framebuffer como textura y escribe el texto del HUD.

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
| `1` a `5` | Elegir planeta destino en el menú |
| `Tab` | Siguiente destino en el menú (incluye el Sol y la Luna) |
| `F` | Warp al destino en el menú |
| `-` / `=` | Velocidad de las órbitas en vivo |
| `I` | Editar la semilla de la próxima partida desde el menú |
| `R` | Generar una semilla aleatoria desde el menú |
| `M` | Métricas de rendimiento |
| `H` | Panel de controles |
| `Esc` | Cerrar el panel; volver al menú desde el juego; salir desde el menú |

En el menú, los planetas tienen el doble de su tamaño normal y sus órbitas están separadas 3×. Núcleo y Droide reciben además 1.5× de tamaño visual, pero conservan el mismo radio de captura de órbita. La órbita de inspección se activa automáticamente al acercarse lo suficiente a un planeta y la cámara transiciona suavemente hasta apuntar a su centro. Dentro de ella, `A` / `D` rodean el planeta, `W` acerca la nave y `S` la aleja; continuar alejándose libera la órbita. El Sol es atravesable: entrar profundamente en él produce un destello blanco e inicia el modo juego.

La semilla mostrada en la esquina superior izquierda determina la generación procedural de la partida. En el menú, `I` permite escribir una semilla numérica y confirmarla con `Enter`; `Backspace` borra y `Esc` cancela la edición. `R` prepara una semilla aleatoria. Reutilizar el mismo número reproduce la misma distribución procedural.

Durante el juego desaparece el sistema solar fijo del menú y permanece únicamente el Sol central, que comienza con cinco veces su tamaño estándar. Los planetas, recompensas y enemigos de la partida serán generados por los sistemas roguelike en lugar de reutilizar los cuerpos decorativos del menú. El mouse tiene una sensibilidad mayor para que los cambios de dirección respondan con menos recorrido.

La partida dura 15 minutos y el reloj permanece centrado en la parte superior. Durante ese tiempo, el Sol crece lentamente y acelera su expansión al acercarse el colapso. El Sol permanece inmóvil mientras la nave esté dentro de 300 unidades; fuera de esa zona se acerca de forma gradual y acelera únicamente a distancias extremas. La cámara mantiene margen suficiente para evitar que desaparezca durante vuelos prolongados con turbo.

Cada partida inicia con 100 puntos de vida, nivel 1 y una barra de experiencia vacía. Vida, nivel y progreso de XP permanecen visibles sobre la barra inferior. La experiencia requerida aumenta con cada nivel; estas estadísticas se reinician al comenzar una nueva partida.

Durante la partida aparece un único tipo de nave enemiga usando el modelo `ENEMY1.obj`. Las naves se generan detrás del jugador, lo persiguen y causan daño por contacto; el choque empuja en direcciones opuestas tanto a la nave como al enemigo y después concede un breve periodo de invulnerabilidad. La salud máxima de todos los enemigos aumenta con el nivel del jugador y conserva proporcionalmente cualquier daño que ya hayan recibido. Las naves se separan entre sí, no pueden ocupar el volumen del jugador y respetan los colliders del Sol y del escenario.

El director de hordas aumenta continuamente la presión durante los 15 minutos: reduce el intervalo entre apariciones y eleva el límite de enemigos activos desde 6 hasta un máximo de 48. Este escalado solo cambia la cantidad; la salud de cada enemigo continúa dependiendo exclusivamente del nivel del jugador.

La nave comienza con un arma normal automática que dispara líneas rojas en la dirección del crosshair. Cada disparo alterna entre los cilindros de las alas, causa 10 puntos de daño, empuja al objetivo con knockback y los enemigos iniciales requieren dos impactos. Los rayos viajan más rápido que la velocidad turbo de la nave, usan colisión continua para no atravesar objetivos entre pasos de simulación y desaparecen al golpear un enemigo, el Sol u otro collider.

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

1. Al iniciar, explorar el sistema ampliado del menú y acercarse a un planeta para activar su órbita automática.
2. Salir de la órbita con una nueva orden de movimiento y volar dentro del Sol para iniciar el juego.
3. Tras el destello blanco, comprobar que el sistema del menú desaparece y solo permanece el Sol.
4. Verificar que el reloj de 15 minutos aparece centrado en la parte superior.
5. Alejarse con turbo para observar el seguimiento gradual del Sol.

## Integrantes

- Pendiente
- Pendiente

## Contribuciones

- Pendiente

## Uso de herramientas de apoyo

- Pendiente
