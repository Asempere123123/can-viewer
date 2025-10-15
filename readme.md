# Como ejecutar
Necesitas trunk para abrirlo. Es un cli que compila un projecto de rust a wasm y le añade el html, bindings de js y todo lo que necesita para poderse abrir mas facil.
```
cargo install trunk
```
Y luego
```
trunk serve
```
Que abre un servidor pequeñito que sirve la pagina y configura hot reloading y mas cosas
# Donde esta el código que hay que cambiar
```
/src/app.rs
```
Lo interesante es cambiar el metodo update de App y lo que contiene el struct
