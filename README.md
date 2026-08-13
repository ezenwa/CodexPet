# Codex Pet para Windows

El instalador deja un observador ligero iniciado con Windows. Cuando detecta que se ejecutó `codex` desde cualquier terminal o desde VS Code, abre una sola mascota y la mantiene visible mientras exista al menos un proceso de Codex activo.

Codex puede crear procesos auxiliares breves durante una tarea. El observador trata todos los procesos de Codex como un grupo para evitar que la mascota se cierre, reaparezca o parpadee cuando uno de esos auxiliares termina. Al finalizar por completo Codex, la mascota se cierra. Tras reiniciar Windows, el observador se carga de nuevo automáticamente y aplica la misma regla.

- Arrastra con el botón izquierdo para moverla.
- Su última posición se conserva entre cierres, reinicios y actualizaciones.
- Haz doble clic para abrir Windows Terminal con Codex.
- Haz clic derecho para activar el inicio con Windows o cerrarla.

Colores: azul trabajando, amarillo requiere atención, verde terminado, morado en espera y gris desconectado.

Esta edición incluye un selector en el menú contextual para elegir entre las ocho mascotas oficiales disponibles: Codex, BSOD, Dewey, Fireball, Null Signal, Rocky, Seedy y Stacky. La elección se conserva al cerrar y volver a abrir.

## Instalación en otros equipos

Ejecuta `CodexPet-Setup.exe`. Se instala para el usuario actual en `%LOCALAPPDATA%\CodexPet`, crea accesos directos en el Escritorio y el menú Inicio, y añade un acceso de desinstalación. No requiere permisos de administrador.
