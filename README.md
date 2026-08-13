# CodexPet

[English documentation](README.en.md)

Una mascota animada para Codex en Windows. CodexPet detecta la actividad de Codex, refleja su estado en tiempo real y permanece sobre el escritorio sin interrumpir tu trabajo.

> [!IMPORTANT]
> Las mascotas y los recursos visuales oficiales incluidos pertenecen a OpenAI/Codex y a sus respectivos titulares. CodexPet no reclama autoría ni propiedad sobre ellos: es un proyecto independiente y no oficial que ofrece una forma alternativa de mostrarlos en Windows.

![Estados de CodexPet](docs/images/codexpet-states.png)

## Características

- Estados visuales ligados a eventos reales de Codex: espera, trabajo, atención requerida, tarea terminada, bloqueo/error y desconexión.
- Ocho mascotas seleccionables: Codex, BSOD, Dewey, Fireball, Null Signal, Rocky, Seedy y Stacky.
- Una sola ventana estable aunque Codex cree procesos auxiliares temporales.
- Posición y tamaño persistentes entre sesiones y actualizaciones.
- Inicio automático con Windows para el usuario actual.
- Búsqueda manual de nuevas versiones desde GitHub Releases.
- Interfaz disponible en inglés y español, seleccionable desde el menú.
- Instalación por usuario; no requiere privilegios de administrador.

## Requisitos

- Windows 10 u 11.
- Windows PowerShell 5.1 o PowerShell 7.
- Una instalación funcional de Codex que ejecute `codex.exe`.
- Internet únicamente para buscar o descargar actualizaciones.

## Instalación

1. Abre la sección [Releases](https://github.com/ezenwa/CodexPet/releases/latest).
2. Descarga `CodexPet-Setup.exe`.
3. Ejecuta el instalador.

CodexPet se instala en `%LOCALAPPDATA%\CodexPet`, crea accesos directos en el Escritorio y el menú Inicio, y registra su observador en el inicio de Windows.

## Uso

- Arrastra la tarjeta con el botón izquierdo para moverla.
- Cambia su tamaño desde las esquinas.
- Haz doble clic para abrir Windows Terminal con Codex.
- Haz clic derecho para elegir mascota, activar el inicio con Windows, buscar actualizaciones o cerrar CodexPet.

### Estados

| Estado | Color | Significado |
| --- | --- | --- |
| En espera | Morado | Codex está abierto y espera una solicitud. |
| Trabajando | Azul | Hay una tarea activa. |
| Necesita atención | Amarillo | Codex solicita aprobación o información. |
| Tarea terminada | Verde | La tarea finalizó correctamente. |
| Bloqueado | Rojo | La tarea falló o fue abortada. |
| Desconectado | Gris | No se detecta un proceso activo de Codex. |

## Actualizaciones

Abre el menú contextual y selecciona **Buscar actualizaciones**. CodexPet consulta exclusivamente la última versión pública de `ezenwa/CodexPet` mediante la API de GitHub. Si encuentra una versión superior, ofrece abrir la página oficial del release; nunca instala ni ejecuta una descarga automáticamente.

## Privacidad y seguridad

CodexPet procesa localmente los registros de sesión ubicados en `%USERPROFILE%\.codex\sessions` para identificar tipos de evento. No envía prompts, respuestas ni contenido de archivos. La comprobación de actualizaciones solo comunica a GitHub una solicitud HTTPS estándar.

## Atribución

Las mascotas distribuidas con CodexPet son recursos visuales oficiales de Codex. Sus diseños, nombres y archivos originales pertenecen a OpenAI/Codex y a sus respectivos titulares. Este repositorio únicamente los presenta mediante una interfaz de escritorio alternativa; no está afiliado, patrocinado ni respaldado oficialmente por OpenAI.

## Desarrollo

El proyecto está implementado en PowerShell y WPF, sin dependencias externas de ejecución.

```powershell
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File .\CodexPet.ps1
```

Para validar la sintaxis:

```powershell
$tokens = $null
$errors = $null
[System.Management.Automation.Language.Parser]::ParseFile((Resolve-Path '.\CodexPet.ps1'), [ref]$tokens, [ref]$errors) | Out-Null
$errors
```

Para generar `dist\CodexPet-Setup.exe` y `dist\CodexPet-Setup.zip` necesitas [Inno Setup 6](https://jrsoftware.org/isinfo.php):

```powershell
.\Build-Release.ps1 -Version 1.2.0
```

## Arquitectura

- `CodexPet.ps1`: ventana WPF, animaciones, idiomas y actualizaciones.
- `CodexPet-State.ps1`: lector incremental y mapeo de eventos reales de Codex a estados visuales.
- `CodexPet-Watcher.ps1`: detecta el grupo de procesos de Codex y mantiene una sola mascota.
- `installer/CodexPet.iss`: instalación y desinstalación nativas mediante Inno Setup.
- `Stop-CodexPet.ps1`: cierre controlado antes de actualizar o desinstalar.
- `assets/pets`: cuadros de animación de las mascotas.

## Solución de problemas

- Si la mascota no aparece, confirma que `codex.exe` esté en ejecución.
- Si una actualización no puede comprobarse, revisa la conexión a GitHub y vuelve a intentarlo desde el menú contextual.
- El registro del observador está en `%LOCALAPPDATA%\CodexPet\watcher.log`.
- Para reinstalar, ejecuta el instalador más reciente; la posición y la mascota elegida se conservan.

## Versiones

Consulta [CHANGELOG.md](CHANGELOG.md) para conocer los cambios publicados.
