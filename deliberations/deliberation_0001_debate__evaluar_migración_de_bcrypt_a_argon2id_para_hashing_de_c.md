# Deliberación MAGI #0001: Debate: Evaluar migración de BCrypt a Argon2id para hashing de c...

- **Categoría**: TECHNICAL_DEBATE
- **Tipo de Contexto**: DILEMMA
- **Veredicto de Consenso**: **APPROVED_MAJORITY**
- **Resumen**: Trinity Consensus: APPROVED_MAJORITY. Tallies -> [APPROVE: 2, REJECT: 0, NEUTRAL: 1]

---

## Votos de la Trinidad y Argumentos Analíticos

### Nodo: Melchior-1 — Voto: `APPROVE`

- **Nivel de Riesgo**: 5 / 10
- **Confianza**: 75%
- **Modelo**: `deepseek/deepseek-v4.1-flash`
- **Versión del Prompt**: `sha256:ce3f03d7523ea99b`
- **Latencia de Ejecución**: 23190 ms

#### Hallazgos Estructurados:

| Categoría | Severidad | Título | Impacto | Recomendación |
| :--- | :--- | :--- | :--- | :--- |
| `architecture` | **HIGH** | Hashing Argon2id síncrono bloquea el reactor async de Tokio | Argon2id consume CPU y memoria durante decenas o cientos de milisegundos por operación; ejecutado en el hilo del reactor paraliza el scheduler de Tokio (head-of-line blocking) y degrada la latencia de todo el microservicio de autenticación bajo carga. | Encapsular el hashing y la verificación en `tokio::task::spawn_blocking` (o un pool de threads dedicado y de tamaño fijo), dimensionado a la CPU y memoria reales del nodo. |
| `architecture` | **HIGH** | Verificación dual por prefijo PHC obligatoria para migración sin downtime | Sustituir BCrypt por Argon2id sin rama de verificación dual hace fallar la validación de todos los hashes existentes, provocando bloqueo total de autenticación. Si el rehash-on-login no es idempotente, accesos concurrentes del mismo usuario generan carreras de escritura sobre el registro de credenciales. | Verificar según prefijo (`$2*`→BCrypt, `$argon2id$`→Argon2id), rehashear y persistir el nuevo hash solo tras verificación exitosa, con actualización idempotente/atómica (transacción o compare-and-set por versión de hash). |
| `architecture` | **HIGH** | Amplificación de memoria N×m bajo concurrencia: riesgo de agotamiento y OOMKill | Un pico de logins concurrentes puede rebasar el límite del contenedor y provocar el OOMKill del microservicio de autenticación, convirtiéndose en un vector de denegación de servicio amplificado respecto a BCrypt, que no es memory-hard. | Acotar la concurrencia de hashing con semáforo o pool de tamaño fijo, alinear `limits.memory` del contenedor con m×concurrencia_máxima más margen, y aplicar rate-limiting/captcha en el endpoint de login. |
| `pragmatism` | **MEDIUM** | Parámetros m/t/p deben calibrarse por benchmark, no asumirse | Parámetros por defecto o copiados pueden ser demasiado bajos (debilita la resistencia buscada, CWE-916) o demasiado altos (latencia inaceptable y superficie de DoS por memoria). | Medir el tiempo de hash en el hardware objetivo, fijar un objetivo (~0.5–1 s o la recomendación OWASP vigente), documentar y versionar los parámetros junto con la configuración del servicio. |
| `architecture` | **MEDIUM** | Estado heterogéneo de hashes persistente sin política de rotación | Los usuarios con baja actividad conservan BCrypt de forma indefinida, manteniendo la deuda y dificultando auditar qué fracción de credenciales ya cumple el nuevo esquema. | Instrumentar telemetría de proporción de hashes por algoritmo e implantar una política de rotación forzada/expiración (o migración por lotes) con objetivo de cobertura medible. |
| `pragmatism` | **INFO** | Coste de integración bajo en Rust y motivación algorítmica válida | La migración es viable con dependencias mantenidas y sin incremento desproporcionado de carga cognitiva, coincidiendo con la valoración pragmática del par. | Adoptar los crates mantenidos, pinear versiones y auditar la cadena de dependencias; proceder con el plan de migración sujeto a las condiciones anteriores. |

> **Evidencia [1] (Hashing Argon2id síncrono bloquea el reactor async de Tokio)**: Patrón de riesgo: llamada directa a `hash_password(...)`/`verify_password(...)` del crate `argon2` (síncrono, CPU+memoria intensivo) dentro de un `async fn` handler sin `tokio::task::spawn_blocking`.

> **Evidencia [2] (Verificación dual por prefijo PHC obligatoria para migración sin downtime)**: Convivencia de formatos: hashes heredados con prefijo `$2b$`/`$2y$` (BCrypt) y nuevos con `$argon2id$`; enrutado de verificación dependiente del prefijo del hash almacenado.

> **Evidencia [3] (Amplificación de memoria N×m bajo concurrencia: riesgo de agotamiento y OOMKill)**: Coste de memoria por verificación ≈ parámetro `m` de Argon2id (típicamente 19–64 MiB); N autenticaciones concurrentes implican N×m de memoria simultánea (CWE-400).

> **Evidencia [4] (Parámetros m/t/p deben calibrarse por benchmark, no asumirse)**: Configuraciones OWASP de referencia (p. ej. m=19 MiB t=2 p=1; m=46 MiB t=1 p=1; alternativas m=64 MiB t=3 p=4); el valor óptimo depende del hardware objetivo.

> **Evidencia [5] (Estado heterogéneo de hashes persistente sin política de rotación)**: Tras rehash-on-login persisten simultáneamente registros `$2b$`/`$2y$` y `$argon2id$` en el almacén de credenciales.

> **Evidencia [6] (Coste de integración bajo en Rust y motivación algorítmica válida)**: Crates `argon2` y `password-hash` maduros, formato PHC estándar y comparación en tiempo constante incluida; BCrypt carece de coste de memoria (CWE-916).


#### Ronda 1: Postura Inicial Independiente
- **Voto Inicial**: `APPROVE` | **Riesgo Inicial**: 5 / 10

La dirección es algorítmicamente correcta y basada en evidencia: Argon2id ganó el Password Hashing Competition (2015) y es la recomendación OWASP para nuevas aplicaciones, siendo memory-hard y resistente a GPU/ASIC, superior a BCrypt (CWE-916). Por ello el objetivo se aprueba. No obstante, el riesgo no reside en la elección del algoritmo sino en la ejecución de la migración, que es donde el análisis no puede confirmarse por falta de código (workspace Rust sin fuente de auth suministrada): (1) la conversión no es directa y exige rehash-on-login con verificación dual enrutada por prefijo PHC; (2) la naturaleza memory-hard introduce un nuevo vector de agotamiento de memoria bajo concurrencia (amplificación N×m), con potencial OOMKill del microservicio; (3) los parámetros m/t/p deben fijarse y calibrarse por benchmark en hardware objetivo, no asumirse. Aprobación condicionada a que se resuelvan estos puntos con verificación reproducible; sin la implementación real, la confianza se limita a 0.72 y el riesgo se sitúa en 5/10.

#### Ronda 2: Dictamen Final tras Debate Cruzado
- **Voto Final**: `APPROVE` | **Riesgo Final**: 5 / 10

MANTENGO mi voto APPROVE (condicionado) y mi riesgo en 5/10 tras el debate cruzado. Ninguno de los pares aporta evidencia técnica nueva que invalide la decisión: la elección algorítmica

### Nodo: Balthasar-2 — Voto: `NEUTRAL`

- **Nivel de Riesgo**: 3 / 10
- **Confianza**: 56%
- **Modelo**: `openai/gpt-5.1-codex-mini`
- **Versión del Prompt**: `sha256:96b068ab989d176a`
- **Latencia de Ejecución**: 10122 ms
- **Flags CWE Detectados**: CWE-400, CWE-522, CWE-521

#### Hallazgos Estructurados:

| Categoría | Severidad | Título | Impacto | Recomendación |
| :--- | :--- | :--- | :--- | :--- |
| `security` | **MEDIUM** | Need to calibrate Argon2id to prevent resource exhaustion | Un atacante que envíe muchas solicitudes de autenticación simultáneas podría provocar agotamiento de memoria, degradación del servicio o caída del microservicio, afectando disponibilidad de autenticación. | Realizar benchmarks en el hardware objetivo para configurar m/t/p; imponer límites de concurrencia o rate limiting; monitorear el consumo de memoria y probar escenarios de carga antes de la migración. |
| `architecture` | **MEDIUM** | Definir flujo de rehash y compatibilidad durante la migración | Sin una estrategia de doble verificación y rehash, los usuarios con hashes antiguos podrían perder acceso o el proceso de autenticación podría fallar, afectando la disponibilidad y la confianza. | Implementar un mecanismo que interprete el algoritmo del hash actual, permita verificar con BCrypt mientras se rehashea con Argon2id en background y soporte procedimientos de rollback y pruebas incrementales. |

> **Evidencia [1] (Need to calibrate Argon2id to prevent resource exhaustion)**: Argon2id es memory-hard; sin una calibración cuidadosa de parámetros (m/t/p) y límites de concurrencia, cada verificación amplifica el uso de memoria y una oleada de solicitudes autenticadas podría agotar la memoria disponible o detonarse OOMKill.

> **Evidencia [2] (Definir flujo de rehash y compatibilidad durante la migración)**: La transición de BCrypt a Argon2id no puede hacerse de forma instantánea: se requiere detectar la versión del hash (por ejemplo, prefijo PHC) para verificar con el algoritmo correcto y rehashear en el siguiente inicio de sesión.


#### Ronda 1: Postura Inicial Independiente
- **Voto Inicial**: `NEUTRAL` | **Riesgo Inicial**: 3 / 10

Argon2id aporta mayor resistencia frente a ataques de fuerza bruta y contra GPUs/ASICs respecto a BCrypt; sin embargo, su adopción requiere evaluación de costos y pruebas de rendimiento para evitar degradación de servicio, por lo que la decisión de migrar debe planificarse cuidadosamente.

#### Ronda 2: Dictamen Final tras Debate Cruzado
- **Voto Final**: `NEUTRAL` | **Riesgo Final**: 3 / 10

Tras analizar el debate sigo manteniendo una postura neutral: si bien Argon2id mejora la resistencia contra ataques de fuerza bruta y abusos GPU/ASIC, la migración introduce nuevos riesgos de agotamiento de recursos y requiere un plan claro de compatibilidad y rehash para no interrumpir la autenticación existente. No tengo evidencia de implementación concreta ni garantías de mitigaciones, por lo que no puedo aprobar ni rechazar el cambio.

### Nodo: Casper-3 — Voto: `APPROVE`

- **Nivel de Riesgo**: 3 / 10
- **Confianza**: 95%
- **Modelo**: `google/gemini-3.1-flash-lite`
- **Versión del Prompt**: `sha256:4d7b522539ba4f62`
- **Latencia de Ejecución**: 3812 ms
- **Flags CWE Detectados**: CWE-916, CWE-400

#### Hallazgos Estructurados:

| Categoría | Severidad | Título | Impacto | Recomendación |
| :--- | :--- | :--- | :--- | :--- |
| `pragmatism` | **MEDIUM** | Sobrecarga de memoria bajo concurrencia | El ajuste incorrecto de los parámetros de memoria (m) en Argon2id puede causar agotamiento de memoria (OOM) en microservicios con alta concurrencia de login. | Realizar benchmarks de carga en el entorno de despliegue real para calibrar el parámetro 'm' de forma que el consumo de memoria total (n_concurrent * m) no comprometa el contenedor. |
| `architecture` | **MEDIUM** | Estrategia de migración (Rehash-on-login) | La falta de una estrategia de doble verificación durante la transición puede resultar en credenciales bloqueadas para usuarios legítimos si no se maneja el re-hashing transparente. | Implementar un sistema de prefijos de formato PHC (Password Hashing Competition) para identificar y migrar los hashes de BCrypt a Argon2id de manera incremental y transparente al iniciar sesión. |

> **Evidencia [1] (Sobrecarga de memoria bajo concurrencia)**: Migración de BCrypt a Argon2id (memory-hard)

> **Evidencia [2] (Estrategia de migración (Rehash-on-login))**: Transición de algoritmo de hashing


#### Ronda 1: Postura Inicial Independiente
- **Voto Inicial**: `APPROVE` | **Riesgo Inicial**: 2 / 10

La migración a Argon2id es una mejora pragmática y estándar de la industria que mejora significativamente la resistencia frente a ataques de fuerza bruta basados en GPU/ASIC con un costo de implementación manejable. En el ecosistema Rust, existen crates maduros (como `argon2`) que facilitan la integración sin introducir dependencias externas complejas ni incrementar innecesariamente la carga cognitiva del equipo.

#### Ronda 2: Dictamen Final tras Debate Cruzado
- **Voto Final**: `APPROVE` | **Riesgo Final**: 3 / 10

Mantengo mi voto de APROBACIÓN. Tras considerar las advertencias de Melchior-1 sobre los riesgos operativos (OOM y calibración), confirmo que estos son retos de implementación gestionables y no fallos de arquitectura. La migración a Argon2id es el estándar de la industria y la implementación en Rust es sumamente ergonómica gracias a crates como `argon2`. La complejidad de la migración (re-hash on login) es una tarea estándar que no justifica rechazar una mejora de seguridad necesaria. Mi enfoque sigue siendo el equilibrio: la seguridad mejorada compensa con creces el esfuerzo de calibración de parámetros que, al realizarse una sola vez por entorno, no supone una carga cognitiva continua para el equipo de desarrollo.

---

## Contexto / Código Fuente Evaluado

```text
Evaluar migración de BCrypt a Argon2id para hashing de credenciales en un microservicio de autenticación
```
