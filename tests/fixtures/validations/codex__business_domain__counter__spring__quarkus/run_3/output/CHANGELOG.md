# Migration Changelog

## [2025-11-27T01:31:16Z] [info] Scanned repository for Spring usage
- Detected Spring Boot parent, Thymeleaf, Web MVC, and test deps in `pom.xml`.
- Found Spring `@Controller`, `@Service`, `@SpringBootApplication`, and Thymeleaf templates.
- Config keys `spring.application.name` and `server.servlet.contextPath` in `src/main/resources/application.properties`.
## [2025-11-27T01:31:23Z] [info] Updated Maven build to Quarkus
- Removed Spring Boot parent and plugin.
- Added Quarkus BOM, `quarkus-maven-plugin`, and dependencies: `quarkus-resteasy`, `quarkus-qute`, `quarkus-arc`, test deps.
- Set Java 17 source/target.
## [2025-11-27T01:31:28Z] [info] Refactored Java code to Quarkus
- Deleted Spring Boot application class and MVC controller; added JAX-RS `CounterResource`.
- Converted `CounterService` to Quarkus CDI (`@ApplicationScoped`) and injection via `@Inject`.
- Adjusted context path to Quarkus config.
## [2025-11-27T01:31:31Z] [info] Migrated templates to Qute and static resources
- Removed Thymeleaf templates (`template.html`).
- Converted `index.html` to Qute syntax and wired up via `Template` injection.
- Added static CSS under `META-INF/resources/css/default.css`.
## [2025-11-27T01:31:36Z] [info] Updated configuration and docker-compose
- Replaced Spring properties with Quarkus `quarkus.http.root-path=/counter`.
- Updated `docker-compose.yml` service name from `counter-spring` to `counter-quarkus` and removed Spring-specific env.
## [2025-11-27T01:31:59Z] [error] Maven build failed due to network and version resolution
- Details: Network access restricted; unable to fetch Quarkus BOM from Maven Central. Also dependency versions appear missing because BOM wasn't imported.
- Impact: Compilation cannot proceed without resolving dependencies.
- Action: Use offline repository or provide local `.m2repo` pre-populated with Quarkus artifacts. Alternatively, pin explicit versions for dependencies and vendor artifacts locally.
