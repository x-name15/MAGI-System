# MAGI SYSTEM — DOCUMENTO MAESTRO DE ARQUITECTURA

## 1. Visión General del Sistema

**MAGI System** es un motor CLI distribuido y orientado a eventos (*event-driven*) diseñado para la auditoría de código, evaluación de arquitecturas de software y análisis de seguridad mediante deliberación multi-agente basada en Grandes Modelos de Lenguaje (LLMs).

Inspirado en el superordenador MAGI del universo *Neon Genesis Evangelion*, el sistema somete cada propuesta de código o decisión técnica al escrutinio paralelo de tres nodos con sesgos analíticos complementarios.

### La Trinidad de Nodos
* **Melchior-1 (El Científico / Lógica & Arquitectura):**
  * *Enfoque:* Calidad de código, patrones de diseño, mantenibilidad, escalabilidad, rendimiento y complejidad ciclomática.
  * *Modelo habitual:* Claude 3.5 Sonnet / DeepSeek-R1.
* **Balthasar-2 (La Madre / Seguridad & Gestión de Riesgo):**
  * *Enfoque:* Auditoría de seguridad, OWASP Top 10, detección de CWEs (Common Weakness Enumeration), sanitización de entradas, exposición de secretos y gestión de permisos.
  * *Poder especial:* **Capacidad de VETO sobre la deliberación**.
  * *Modelo habitual:* GPT-4o / Gemini Pro.
* **Casper-3 (La Persona / Pragmatismo & Experiencia de Desarrollo - DX):**
  * *Enfoque:* Viabilidad real de implementación, impacto en la curva de aprendizaje del equipo, sobreingeniería (*overengineering*), costo computacional y simplicidad.
  * *Modelo habitual:* Llama 3 / Mistral (vía Ollama u otros proveedores).

---

## 2. Arquitectura del Sistema

El sistema utiliza una arquitectura desvinculada (*decoupled*) basada en **SpacetimeDB**, dividida en dos componentes principales dentro de un Cargo Workspace en Rust.

```text
┌────────────────────────────────────────────────────────────────────────┐
│                              CLIENTE CLI                               │
│                                                                        │
│  ┌────────────────┐     ┌──────────────────┐     ┌──────────────────┐  │
│  │   Melchior-1   │     │   Balthasar-2    │     │     Casper-3     │  │
│  │ (Anthropic/    │     │ (OpenAI API /    │     │  (Ollama Local / │  │
│  │ Claude 3.5)    │     │     GPT-4o)      │     │  Google Gemini)  │  │
│  └───────┬────────┘     └────────┬─────────┘     └────────┬─────────┘  │
│          │                       │                        │            │
│          └───────────────────────┼────────────────────────┘            │
│                                  │ (concurrent cancellable requests)   │
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
1. **Invocación:** El desarrollador ejecuta `magi audit src/auth.rs --prompt "Revisa la implementación del middleware JWT"`.
2. **Creación de Estado:** El CLI se conecta a SpacetimeDB y ejecuta el reducer `create_deliberation`. La DB asigna un `id` y fija el estado en `PENDING`.
3. **Suscripción Pub/Sub:** El CLI escucha cambios en tiempo real en las tablas `deliberations` y `node_votes` vía WebSockets.
4. **Ronda Inicial Concurrente:** El orquestador ejecuta una evaluación paralela y cancelable para Melchior, Balthasar y Casper.
5. **Debate de la Trinidad:** Cada nodo recibe las posiciones iniciales de los otros nodos, cuestiona sus argumentos y emite una posición final.
6. **Consenso Automático:** Cuando `node_votes` registra los 3 votos finales para un `deliberation_id`, el motor interno de SpacetimeDB ejecuta `eval_consensus`, aplicando reglas de votación y el Veto de Balthasar.
7. **Renderizado NERV TUI:** La TUI basada en `ratatui` y `crossterm` conserva el historial de resultados, argumentos y veredictos dentro de la pantalla interactiva.

---

## 3. Stack Tecnológico & Dependencias

### Servidor (SpacetimeDB Engine)
* **Lenguaje:** Rust (`wasm32-unknown-unknown`).
* **Crate Core:** `spacetimedb` (Esquemas de tablas, índices, llaves primarias y reducers transaccionales).

### Cliente (CLI Orchestrator)
* **Lenguaje:** Rust (Edición 2021).
* **Runtime Asíncrono:** `tokio` (con características `full` para concurrencia HTTP y temporizadores).
* **Cliente HTTP:** `reqwest` (Soporte JSON para APIs de OpenAI, Anthropic, Google u Ollama).
* **CLI Parser:** `clap` (Subcomandos, argumentos y banderas).
* **Serialización:** `serde`, `serde_json`.
* **Interfaz de Usuario / TUI:** `colored` (Colores ANSI), `indicatif` (Spinners), `comfy-table` (Renderizado de tablas).
* **SpacetimeDB SDK:** `spacetimedb-sdk` (Cliente WebSocket en tiempo real).

---

## 4. Estructura del Cargo Workspace

```text
magi-system/
├── Cargo.toml                    # Manifest de la raíz del Workspace
├── README.md
├── server/                       # Módulo Wasm desplegado DENTRO de SpacetimeDB
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                # Modelos de tablas, reducers y motor de consenso
└── client/                       # CLI binario ejecutable
    ├── Cargo.toml
    └── src/
        ├── main.rs               # Entrypoint & CLI Argument Parsing (clap)
        ├── config.rs             # Carga de API Keys y variables de entorno (.env)
        ├── core/
        │   ├── mod.rs
        │   └── orchestrator.rs   # Coordinación de rondas concurrentes y debate
        ├── llm/
        │   ├── mod.rs            # Trait común `LlmProvider`
        │   ├── melchior.rs       # Persona científica y despacho de proveedor
        │   ├── balthasar.rs      # Persona de seguridad y Veto
        │   ├── casper.rs         # Persona pragmática y despacho de proveedor
        │   └── mock.rs           # Proveedor determinista offline
        ├── db/
        │   ├── mod.rs
        │   └── client.rs         # Conexión, reducers y suscripciones a SpacetimeDB
        └── ui/
            ├── mod.rs
            └── nerv_theme.rs     # Formateador visual NERV para terminal
```

---

## 5. Modelo de Datos de SpacetimeDB

### Tabla: `deliberation`
* **`id`**: `u64` *(Primary Key / Autoinc)* — ID único del debate.
* **`author`**: `String` *(Indexed)* — Usuario o sistema origen (ej: `"felix@laptop"`).
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

El reducer de consenso se ejecuta de manera determinista e inmutable dentro de SpacetimeDB al recibir el tercer voto.

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

### Posibles Veredictos Finales:
1. `VETO_BALTHASAR_SECURITY`: Activado si Balthasar-2 emite `REJECT` con `risk_score >= 8`. Anula cualquier voto a favor de Melchior y Casper.
2. `APPROVED_UNANIMOUS`: 3-0 a favor. La propuesta cumple con arquitectura, seguridad y pragmatismo.
3. `APPROVED_MAJORITY`: 2-1 a favor. La propuesta es aprobada con advertencias menores.
4. `REJECTED_MAJORITY`: 1-2 en contra. Se requiere refactorización.
5. `REJECTED_UNANIMOUS`: 0-3 en contra. Rechazo total por los 3 nodos.

---

## 7. Plan de Implementación Progresivo

1. **Fase 1: Server Module (SpacetimeDB)**
   * Definición de structs de tablas y reducers en `server/src/lib.rs`.
   * Implementación del motor `eval_consensus`.
   * Publicación y despliegue local en la instancia de SpacetimeDB.
2. **Fase 2: Adapters LLM & Cliente CLI Base**
   * Configuración de banderas CLI con `clap`.
  * Implementación del despacho HTTP con `reqwest` y rondas concurrentes cancelables.
3. **Fase 3: Integración SDK SpacetimeDB & Event Loop**
   * Conexión WebSocket, llamadas a reducers y suscripción a eventos.
4. **Fase 4: Terminal UI (NERV Theme)**
   * Formateador estético para la terminal con reporte de CWEs, tiempos de respuesta y veredicto final.