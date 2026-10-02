# MAGI SYSTEM — DOCUMENTO MAESTRO DE ARQUITECTURA 🧠

## 1. Visión General del Sistema

**MAGI System** es un motor CLI distribuido y orientado a eventos (*event-driven*) diseñado para la auditoría de código, evaluación de arquitecturas de software y análisis de seguridad mediante deliberación multi-agente basada en Grandes Modelos de Lenguaje (LLMs).

Inspirado en el superordenador MAGI del universo *Neon Genesis Evangelion*, el sistema somete cada propuesta de código o decisión técnica al escrutinio paralelo de tres nodos con sesgos analíticos complementarios.

### La Trinidad de Nodos
* **Melchior-1 (El Científico / Lógica & Arquitectura):**
  * *Enfoque:* Calidad de código, patrones de diseño, mantenibilidad, escalabilidad, rendimiento y complejidad ciclomática.
  * *Arquetipo:* Rigor matemático, análisis de algoritmos y eliminación de antipatrones.
* **Balthasar-2 (La Madre / Seguridad & Gestión de Riesgo):**
  * *Enfoque:* Auditoría de seguridad, OWASP Top 10, detección de CWEs (Common Weakness Enumeration), sanitización de entradas, exposición de secretos y gestión de permisos.
  * *Poder especial:* **Capacidad de VETO UNILATERAL sobre la deliberación (`REJECT` con riesgo $\ge 8$)**.
* **Casper-3 (La Mujer / Pragmatismo & Experiencia de Desarrollo - DX):**
  * *Enfoque:* Viabilidad real de implementación, impacto en la curva de aprendizaje del equipo, sobreingeniería (*overengineering*), costo computacional y simplicidad.
  * *Regla estricta:* Cero tolerancia a complejidad o daemons innecesarios para herramientas locales.

---

## 2. Arquitectura del Sistema

El sistema utiliza una arquitectura desacoplada (*decoupled*) basada en **SpacetimeDB**, dividida en dos componentes principales dentro de un Cargo Workspace en Rust.

```text
┌────────────────────────────────────────────────────────────────────────┐
│                              CLIENTE CLI / TUI                         │
│                                                                        │
│  ┌────────────────┐     ┌──────────────────┐     ┌──────────────────┐  │
│  │   Melchior-1   │     │   Balthasar-2    │     │     Casper-3     │  │
│  │  (Arquitectura)│     │(Seguridad & Veto)│     │(Pragmatismo & DX)│  │
│  └───────┬────────┘     └────────┬─────────┘     └────────┬─────────┘  │
│          │                       │                        │            │
│          └───────────────────────┼────────────────────────┘            │
│                                  │ (2 Rondas de debate concurrente)    │
│                                  ▼                                     │
│                     ┌────────────────────────┐                         │
│                     │  Orquestador Async CLI │                         │
│                     └───────────┬────────────┘                         │
└─────────────────────────────────┼──────────────────────────────────────┘
                                  │
                 Suscripción /    │  Llamada a Reducers
                   WebSockets     │  (create_deliberation, submit_node_vote)
                                  ▼
┌────────────────────────────────────────────────────────────────────────┐
│                       SPACETIMEDB ENGINE (WASM)                        │
│                                                                        │
│  ┌─────────────────┐    ┌─────────────────┐    ┌────────────────────┐  │
│  │  deliberations  │    │   node_votes    │    │  consensus_results │  │
│  └────────┬────────┘    └────────┬────────┘    └─────────▲──────────┘  │
│           │                      │                       │             │
│           └──────────────────────┴───────────────────────┘             │
│                                  │                                     │
│                                  ▼                                     │
│                     [ Reducer: eval_consensus ]                        │
└────────────────────────────────────────────────────────────────────────┘
```

### Flujo de Datos (Lifecycle de una Consulta)
1. **Invocación:** El desarrollador ejecuta `magi idea <file.md>`, `magi maintain <file>`, `magi triage <log>` o usa la consola interactiva `magi tui`.
2. **Creación de Estado:** El CLI se conecta a SpacetimeDB y ejecuta el reducer `create_deliberation`. La DB asigna un `id` y fija el estado en `PENDING`.
3. **Ronda 1 (Evaluación Ciega):** El orquestador ejecuta una evaluación paralela y cancelable para Melchior, Balthasar y Casper sin que conozcan las posturas ajenas (elimina sesgo de anclaje).
4. **Ronda 2 (Debate de la Trinidad):** Cada nodo recibe las posiciones iniciales de los otros nodos, cuestiona sus argumentos y emite una posición final.
5. **Consenso Automático:** Los votos finales se envían a SpacetimeDB mediante `submit_node_vote`. El reducer `eval_consensus` aplica deterministamente la regla de Veto de Balthasar y mayorías.
6. **Renderizado NERV TUI:** Se despliegan los monitores CRT fosforados NERV y se persiste un reporte Markdown estructurado en `deliberations/`.

---

## 3. Stack Tecnológico & Dependencias

### Servidor (SpacetimeDB Engine)
* **Lenguaje:** Rust (`wasm32-unknown-unknown`).
* **Crate Core:** `spacetimedb` (Esquemas de tablas, índices, llaves primarias y reducers transaccionales).

### Cliente (CLI Orchestrator & TUI)
* **Lenguaje:** Rust (Edición 2021).
* **Runtime Asíncrono:** `tokio` (con características `full` para concurrencia HTTP y temporizadores).
* **Cliente HTTP:** `reqwest` (Soporte JSON para APIs de Google Gemini, OpenAI, Anthropic, Ollama, DeepSeek, Grok).
* **CLI Parser:** `clap` (Subcomandos `idea`, `maintain`, `triage`, argumentos y banderas).
* **Serialización:** `serde`, `serde_json`.
* **Interfaz de Usuario / TUI:** `ratatui` + `crossterm` (consola interactiva) y `colored` / ANSI 24-bit TrueColor (monitores fosforados NERV).
* **SpacetimeDB SDK / HTTP:** Cliente asíncrono WebSocket y REST.

---

## 4. Estructura del Cargo Workspace

```text
magi-system/
├── Cargo.toml                    # Manifest de la raíz del Workspace
├── README.md
├── docker-compose.yml
├── magi.ps1
├── server/                       # Módulo Wasm desplegado DENTRO de SpacetimeDB
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                # Modelos de tablas, reducers y motor de consenso
└── client/                       # CLI binario ejecutable
    ├── Cargo.toml
    ├── skills/                   # Personas desacopladas en Markdown
    │   └── magi-system/
    │       ├── melchior.md
    │       ├── balthasar.md
    │       └── casper.md
    └── src/
        ├── main.rs               # Entrypoint & CLI Argument Parsing (clap)
        ├── config.rs             # Carga dinámica de LLMs y variables (.env)
        ├── core/
        │   ├── mod.rs
        │   └── orchestrator.rs   # Coordinación de 2 rondas y debate de pares
        ├── llm/
        │   ├── mod.rs            # Trait común `LlmProvider` y despachador
        │   ├── melchior.rs       # Arquetipo científico
        │   ├── balthasar.rs      # Arquetipo de seguridad y Veto unilateral
        │   ├── casper.rs         # Arquetipo pragmático y anti-sobreingeniería
        │   └── mock.rs           # Proveedor determinista offline
        ├── db/
        │   ├── mod.rs
        │   └── client.rs         # Conexión, reducers y sincronización SpacetimeDB
        ├── skills/
        │   └── mod.rs            # Cargador dinámico de prompts Markdown
        └── ui/
            ├── mod.rs
            ├── nerv_theme.rs     # Renderizador diegético CRT Phosphor NERV
            ├── report.rs         # Generador de reportes Markdown persistentes
            └── tui.rs            # Consola interactiva NERV en Ratatui/Crossterm
```

---

## 5. Modelo de Datos de SpacetimeDB

### Tabla: `deliberation`
* **`id`**: `u64` *(Primary Key / Autoinc)* — ID único del debate.
* **`author`**: `String` *(Indexed)* — Usuario o sistema origen (ej: `"developer@magi"`).
* **`title`**: `String` — Título o resumen corto de la consulta.
* **`prompt`**: `String` — Instrucción o duda técnica del desarrollador.
* **`context_type`**: `String` — Tipo de adjunto (`"CODE_SNIPPET"`, `"DOCKERFILE"`, `"OPENAPI_SPEC"`, `"NONE"`).
* **`context_payload`**: `String` — Contenido del archivo o código a auditar.
* **`status`**: `String` *(Indexed)* — Estado (`"PENDING"`, `"DEBATING"`, `"RESOLVED"`, `"FAILED"`).
* **`created_at`**: `Timestamp` — Fecha/hora de creación.

### Tabla: `node_vote`
* **`id`**: `u64` *(Primary Key / Autoinc)* — ID único del voto.
* **`deliberation_id`**: `u64` *(Indexed / Foreign Key)* — Referencia a `deliberation.id`.
* **`node_id`**: `String` *(Indexed)* — Nombre del nodo (`"Melchior-1"`, `"Balthasar-2"`, `"Casper-3"`).
* **`argument`**: `String` — Justificación técnica emitida por el LLM.
* **`cwe_flags`**: `String` — Cadena JSON con lista de CWEs detectados (ej: `["CWE-89", "CWE-200"]`).
* **`vote`**: `String` — Postura del nodo (`"APPROVE"`, `"REJECT"`, `"NEUTRAL"`).
* **`risk_score`**: `u8` — Nivel de riesgo percibido (escala de 1 a 10).
* **`execution_time_ms`**: `u32` — Latencia de respuesta en milisegundos.
* **`created_at`**: `Timestamp` — Fecha/hora de emisión.

### Tabla: `consensus_result`
* **`deliberation_id`**: `u64` *(Primary Key / Foreign Key)* — Referencia a `deliberation.id`.
* **`verdict`**: `String` — Resultado final del consenso.
* **`tally_approves`**: `u8` — Total de votos a favor.
* **`tally_rejects`**: `u8` — Total de votos en contra.
* **`tally_neutrals`**: `u8` — Total de abstenciones.
* **`summary`**: `String` — Resumen consolidado del veredicto.
* **`created_at`**: `Timestamp` — Fecha/hora de resolución.

---

## 6. Lógica de Consenso & Veto de Balthasar

```text
┌────────────────────────────────────────────────────────────────────────┐
│                     EVALUACIÓN DE VOTOS RECIBIDOS                      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
                     ¿Balthasar-2 votó REJECT y
                      risk_score >= 8?
                     /                \
                 (SÍ)                  (NO)
                 /                      \
                ▼                        ▼
    ┌──────────────────────┐   Evaluar conteo
    │ VETO_BALTHASAR_RISK  │   de votos:
    └──────────────────────┘   - 3 Approves -> APPROVED_UNANIMOUS
                               - 2 Approves -> APPROVED_MAJORITY
                               - 2 Rejects  -> REJECTED_MAJORITY
                               - 3 Rejects  -> REJECTED_UNANIMOUS
```