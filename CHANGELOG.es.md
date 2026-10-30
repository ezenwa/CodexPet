# Historial de cambios

[Changelog in English](CHANGELOG.md)

## 2.2.0 - 2026-10-30

- Se sustituyó Inno Setup por instalación, migración y desinstalación implementadas en Rust.
- Se incorporaron todos los cuadros de las mascotas al ejecutable portable autónomo.
- Se reemplazaron los scripts PowerShell por Cargo xtask para comprobar, empaquetar y probar instalaciones aisladas.
- Se conservan las preferencias, la elección de inicio con Windows, el icono y el registro de desinstalación al actualizar.
- Se retiraron las definiciones del instalador, auxiliares y copias de recursos instaladas que ya no se utilizan.

## 2.1.1 - 2026-10-30

- Se agregó un icono transparente de la mascota en diez tamaños, de 16 a 256 píxeles.
- El ejecutable de Windows incorpora el icono y la versión definida por Cargo.
- Se aplicó a las ventanas nativas, los accesos del Escritorio/Inicio, el instalador y la entrada de desinstalación.
- Se verifica que el paquete incluya el icono incrustado y los recursos de versión.

## 2.1.0 - 2026-10-30

- Se agregó **Minimizar a la bandeja**, con icono de la mascota, estado localizado y opciones para mostrar o cerrar.
- La mascota permanece minimizada aunque cambie la actividad; el icono se recupera si Explorer se reinicia.
- La integración nativa se dividió en módulos de aplicación, gráficos, procesos, inicio, actualizaciones y bandeja.
- Los scripts reutilizables se agruparon en `tools/` y los auxiliares del instalador en `installer/`, actualizando sus rutas.
- Los 72 PNG de cada mascota quedaron en su carpeta; se eliminaron Sixel sin uso, lanzadores obsoletos, scripts de un solo uso, respaldos y temporales.
- Se documentaron la arquitectura, el desarrollo y la ubicación de archivos nuevos; se unificaron las reglas de texto.
- Las verificaciones eliminan sus temporales y Cargo.toml es la única fuente de la versión.

## 2.0.0 - 2026-10-29

- Se reemplazó la aplicación PowerShell/WPF por un ejecutable nativo Rust/Win32.
- El observador está integrado; cerrar la mascota termina también la supervisión.
- Se conservaron las ocho mascotas, las animaciones y las preferencias anteriores.
- Las preguntas y aprobaciones se relacionan por llamada; varias sesiones participan en el estado.
- La lectura de registros y las actualizaciones se ejecutan fuera del hilo de interfaz.
- Se agregó empaquetado portátil, comprobaciones de ventana nativa y pruebas de regresión.
- Cargo.toml determina la versión de la aplicación y del instalador.

## 1.2.3 - 2026-08-12

- Se agregó la opción localizada **About / Acerca de** al menú contextual.
- El cuadro identifica a Joshua Ezenwa como autor y atribuye las mascotas y recursos visuales oficiales a OpenAI/Codex y sus respectivos titulares.

## 1.2.2 - 2026-08-12

- Detección en tiempo real de autorizaciones pendientes mediante la telemetría SQLite local de Codex.
- Las autorizaciones ya recordadas no activan falsamente el estado **Necesita atención**.
- El tiempo de **Tarea terminada** se calcula desde el evento real, incluso cuando Codex conserva una fecha antigua en el archivo de sesión activo.

## 1.2.1 - 2026-08-12

- El estado **Tarea terminada** permanece visible hasta que comienza otra tarea o transcurren diez minutos de inactividad.
- Prueba automática para impedir que el estado terminado vuelva prematuramente a espera.

## 1.2.0 - 2026-08-12

- Corrección del texto corrupto en **Buscar actualizaciones** bajo Windows PowerShell 5.1.
- Scripts PowerShell normalizados a UTF-8 con BOM para conservar correctamente la ortografía en español.
- Interfaz de CodexPet disponible en inglés y español con cambio de idioma desde el menú.
- Instalador bilingüe migrado de IExpress a Inno Setup 6.
- Detección incremental corregida para sesiones grandes y eventos reales de inicio, atención, finalización y error.

## 1.1.0 - 2026-08-12

- Opción **Buscar actualizaciones** en el menú contextual.
- Comprobación segura de GitHub Releases sin instalación automática.
- Documentación ampliada y preview de los distintos estados.
- Proceso reproducible para construir los instaladores con el nombre CodexPet.

## 1.0.0 - 2026-08-12

- Primera versión pública.
- Estados animados basados en los eventos de sesión de Codex.
- Selector con ocho mascotas.
- Persistencia de posición, tamaño y personaje elegido.
- Observador estable para procesos principales y auxiliares de Codex.
