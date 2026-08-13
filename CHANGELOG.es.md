# Historial de cambios

[Changelog in English](CHANGELOG.md)

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
