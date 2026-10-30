# CodexPet

[English documentation](README.md)

Una mascota de escritorio para Windows escrita en Rust. Representa la actividad local de Codex en una ventana transparente que permanece sobre el escritorio.

> Las mascotas y recursos visuales oficiales pertenecen a OpenAI/Codex y a sus respectivos titulares. CodexPet es un proyecto independiente y no oficial.

![Estados de CodexPet](docs/images/codexpet-states.png)

## Funciones

- Ocho mascotas y seis estados: espera, trabajo, atención, finalización, fallo y desconexión.
- Español e inglés; mascota, tamaño y posición persistentes.
- Ventana Win32 y observador en segundo plano dentro de un único proceso.
- Icono de la mascota incrustado en el ejecutable y el instalador, con tamaños para el escalado de Windows.
- **Minimizar a la bandeja** desde el menú contextual. Para volver, pulsa el icono o selecciona **Mostrar mascota**. La supervisión continúa mientras está oculta.
- Varias sesiones participan en el estado; las preguntas y aprobaciones se relacionan por llamada.
- Instalación por usuario, inicio con Windows y búsqueda manual de actualizaciones.

## Instalación y uso

Requiere Windows 10 versión 1703 o posterior, o Windows 11, de 64 bits. Los estados necesitan `codex.exe` y sus registros locales de sesión.

Ejecuta `CodexPet-Setup.exe`, o extrae todo `CodexPet-Portable.zip` y abre `CodexPet/CodexPet.exe`, conservando la carpeta `assets` junto al ejecutable. Se mantienen las preferencias de `%APPDATA%\CodexPet`.

- Arrastra la tarjeta para moverla y las esquinas para cambiar su tamaño.
- El clic derecho abre las opciones de mascota, idioma, bandeja, inicio, actualizaciones y cierre.
- El doble clic abre Windows Terminal con Codex.
- Cerrar la mascota termina también el observador.

| Comando | Función |
| --- | --- |
| `CodexPet.exe` | Muestra la mascota, incluso cuando Codex está desconectado. |
| `CodexPet.exe --watch` | La muestra mientras Codex está activo y permanece en segundo plano al salir. |
| `CodexPet.exe --stop` | Solicita el cierre normal de la instancia nativa. |

La minimización manual se mantiene hasta que restaures la mascota. El menú del icono también permite **Cerrar mascota**. Windows puede colocar el icono dentro del menú de iconos ocultos de la bandeja.

## Privacidad

Los registros JSONL y los metadatos de aprobación se leen localmente desde `$CODEX_HOME`, por defecto `%USERPROFILE%\.codex`; SQLite se consulta en modo de solo lectura. Los datos permanecen en el equipo. Solo la búsqueda de actualizaciones solicitada por el usuario contacta con GitHub y nunca instala descargas automáticamente. Los formatos internos de Codex pueden cambiar; consulta la [arquitectura y sus limitaciones](docs/ARCHITECTURE.md).

## Desarrollo y ubicación de archivos nuevos

Se aplican convenciones habituales de proyectos Cargo. Consulta la [guía de desarrollo y empaquetado](docs/DEVELOPMENT.md) y la [arquitectura](docs/ARCHITECTURE.md).

| Tipo de archivo nuevo | Ubicación |
| --- | --- |
| Lógica de la aplicación en Rust | `src/`, agrupada por responsabilidad. |
| Integración con Windows | El módulo correspondiente de `src/platform/`. |
| Pruebas unitarias | Junto al módulo en `#[cfg(test)]`; crear `tests/` cuando existan pruebas de integración reales. |
| Cuadros utilizados por la mascota | `assets/pets/<mascota>/frame_000.png` hasta `frame_071.png`. |
| Icono incrustado de la aplicación | `assets/icons/codexpet.ico`; `build.rs` lo incorpora al compilar. |
| Documentación técnica o capturas | `docs/` o `docs/images/`. |
| Scripts de compilación, preparación y verificación | `tools/`. |
| Definiciones y auxiliares del instalador | `installer/`. |

`Cargo.toml` determina la versión y `Cargo.lock` conserva la resolución de dependencias. `.build/`, `target/` y `dist/` contienen herramientas locales o resultados generados y están excluidos de Git. Los secretos y ajustes particulares del equipo no deben incorporarse al repositorio.

Consulta [CHANGELOG.es.md](CHANGELOG.es.md) para conocer los cambios.
